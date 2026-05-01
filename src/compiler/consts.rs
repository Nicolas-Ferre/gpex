use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::types::StructDefinition;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct ConstChecker<'item, 'index> {
    is_in_const_fn: bool,
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> ConstChecker<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            is_in_const_fn: false,
            indexes,
        }
    }

    pub(crate) fn set_is_in_const_fn(&mut self, is_in_const_fn: bool) {
        self.is_in_const_fn = is_in_const_fn;
    }

    pub(crate) fn is_expr_const(&self, node: &Expr) -> bool {
        match node {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_) => true,
            Expr::Call(node) => self.is_call_const(node),
            Expr::Ident(node) => self.is_ident_const(node),
        }
    }

    pub(crate) fn is_item_const(&self, node: ItemRef<'_>) -> bool {
        match node {
            ItemRef::Var(_) => false,
            ItemRef::Const(_) | ItemRef::Struct(_) => true,
            ItemRef::Fn(node) => node.const_keyword_span.is_some(),
            ItemRef::Param(_) => self.is_in_const_fn,
        }
    }

    fn is_ident_const(&self, node: &Ident) -> bool {
        self.indexes
            .sources
            .get(&node.id)
            .is_some_and(|source| self.is_item_const(*source))
    }

    fn is_call_const(&self, node: &Call) -> bool {
        self.indexes
            .sources
            .get(&node.id)
            .is_some_and(|source| self.is_item_const(*source))
            && node.args.iter().all(|arg| self.is_expr_const(arg))
    }
}

#[derive(Debug)]
pub(crate) struct ConstResolver<'item, 'index> {
    scope_values: Vec<HashMap<u64, ConstValue<'item>>>,
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> ConstResolver<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            scope_values: vec![],
            indexes,
        }
    }

    pub(crate) fn expr_value(&mut self, expr: &Expr) -> ConstValue<'item> {
        match expr {
            Expr::F32Literal(node) => Self::f32_literal_value(node),
            Expr::U32Literal(node) => Self::u32_literal_value(node),
            Expr::I32Literal(node) => Self::i32_literal_value(node),
            Expr::BoolLiteral(node) => ConstValue::Bool(node.value),
            Expr::Call(node) => self.call_value(node),
            Expr::Ident(node) => self.ident_value(node),
        }
    }

    fn i32_literal_value(node: &I32Literal) -> ConstValue<'static> {
        if let Some(value) = node.value {
            ConstValue::I32(value)
        } else {
            ConstValue::Unknown
        }
    }

    fn u32_literal_value(node: &U32Literal) -> ConstValue<'static> {
        if let Some(value) = node.value {
            ConstValue::U32(value)
        } else {
            ConstValue::Unknown
        }
    }

    fn f32_literal_value(node: &F32Literal) -> ConstValue<'static> {
        if let Some(value) = node.value {
            ConstValue::F32(value)
        } else {
            ConstValue::Unknown
        }
    }

    fn ident_value(&mut self, node: &Ident) -> ConstValue<'item> {
        match self.indexes.sources.get(&node.id) {
            Some(ItemRef::Var(_)) => ConstValue::RuntimeValue,
            Some(ItemRef::Const(child)) => self.expr_value(&child.value),
            Some(ItemRef::Struct(child)) => ConstValue::TypeRef(child),
            Some(ItemRef::Param(child)) => self.value(child.id),
            Some(ItemRef::Fn(_)) => unreachable!("identifier should not refer to a function"),
            None => ConstValue::Unknown,
        }
    }

    fn call_value(&mut self, node: &Call) -> ConstValue<'item> {
        match self.indexes.sources.get(&node.id) {
            Some(ItemRef::Fn(source)) => self.fn_call_value(node, source),
            Some(ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_)) => {
                unreachable!("identifier should not refer to a value")
            }
            None => ConstValue::Unknown,
        }
    }

    fn fn_call_value(&mut self, node: &Call, source: &FnDefinition) -> ConstValue<'item> {
        let param_args = node
            .args
            .iter()
            .zip(&source.params.params)
            .map(|(arg, param)| (param.id, self.expr_value(arg)))
            .collect::<Vec<_>>();
        self.run_scoped(|self_| {
            for (param_id, arg_value) in param_args {
                match arg_value {
                    ConstValue::TypeRef(_)
                    | ConstValue::I32(_)
                    | ConstValue::U32(_)
                    | ConstValue::F32(_)
                    | ConstValue::Bool(_) => self_.add_value(param_id, arg_value),
                    ConstValue::Unknown | ConstValue::RuntimeValue => return arg_value,
                }
            }
            self_.fn_value(source)
        })
    }

    fn fn_value(&mut self, node: &FnDefinition) -> ConstValue<'item> {
        if node.const_keyword_span.is_none() {
            ConstValue::RuntimeValue
        } else if let Some(return_) = node.return_statement() {
            self.expr_value(&return_.value)
        } else {
            ConstValue::Unknown
        }
    }

    fn run_scoped<O>(&mut self, callback: impl FnOnce(&mut Self) -> O) -> O {
        self.scope_values.push(HashMap::new());
        let output = callback(self);
        self.scope_values.pop();
        output
    }

    fn value(&self, id: u64) -> ConstValue<'item> {
        self.scope_values
            .last()
            .and_then(|values| values.get(&id))
            .cloned()
            .unwrap_or(ConstValue::RuntimeValue)
    }

    fn add_value(&mut self, id: u64, value: ConstValue<'item>) {
        let current_scope_index = self.scope_values.len() - 1;
        self.scope_values[current_scope_index].insert(id, value);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstValue<'item> {
    TypeRef(&'item StructDefinition),
    I32(i32),
    U32(u32),
    F32(f32),
    Bool(bool),
    Unknown,
    RuntimeValue,
}

impl<'item> ConstValue<'item> {
    pub(crate) fn type_ref(&self) -> Option<&'item StructDefinition> {
        if let Self::TypeRef(type_) = self {
            Some(type_)
        } else {
            None
        }
    }
}
