use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, Statement};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub(crate) struct ConstChecker<'item, 'index> {
    pub(crate) location: ConstLocation,
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> ConstChecker<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            location: ConstLocation::Other,
            indexes,
        }
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
            ItemRef::Param(node) => match self.location {
                ConstLocation::FnSignature | ConstLocation::ConstCallArg => {
                    node.const_mark_span().is_some()
                }
                ConstLocation::ConstFnBody => true,
                ConstLocation::Other => false,
            },
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConstLocation {
    FnSignature,
    ConstFnBody,
    ConstCallArg,
    Other,
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

    pub(crate) fn enter_scope(&mut self) {
        self.scope_values.push(HashMap::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scope_values.pop();
    }

    pub(crate) fn value(&self, id: u64) -> ConstValue<'item> {
        self.scope_values
            .last()
            .and_then(|values| values.get(&id))
            .cloned()
            .unwrap_or(ConstValue::RuntimeValue)
    }

    pub(crate) fn add_value(&mut self, id: u64, value: ConstValue<'item>) {
        let current_scope_index = self.scope_values.len() - 1;
        self.scope_values[current_scope_index].insert(id, value);
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
            ConstValue::F32(HashableF32(value))
        } else {
            ConstValue::Unknown
        }
    }

    fn ident_value(&mut self, node: &Ident) -> ConstValue<'item> {
        match self.indexes.sources.get(&node.id) {
            Some(ItemRef::Var(_)) => ConstValue::RuntimeValue,
            Some(ItemRef::Const(child)) => self.expr_value(&child.value),
            Some(ItemRef::Struct(child)) => ConstValue::TypeRef(child),
            Some(ItemRef::Param(child)) => {
                let value = self.value(child.id);
                if value == ConstValue::RuntimeValue && child.const_mark_span().is_some() {
                    ConstValue::Param(child)
                } else {
                    value
                }
            }
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
        debug_assert_eq!(node.args.len(), source.params.params.len());
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
                    | ConstValue::Param(_)
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
            return ConstValue::RuntimeValue;
        }
        match &node.body {
            FnBody::Compilerimpl(_) => self.fn_compilerimpl_value(node),
            FnBody::Statements(body) => self.fn_body_value(body),
        }
    }

    fn fn_compilerimpl_value(&self, node: &FnDefinition) -> ConstValue<'item> {
        if node.name == "__add__" {
            let left = self.value(node.params.params[0].id);
            let right = self.value(node.params.params[1].id);
            match (left, right) {
                (ConstValue::I32(left), ConstValue::I32(right)) => {
                    ConstValue::I32(left.wrapping_add(right))
                }
                _ => unreachable!("not implemented `{}` constant GPU function", node.name),
            }
        } else {
            unreachable!("not implemented `{}` constant GPU function", node.name)
        }
    }

    fn fn_body_value(&mut self, body: &FnStatementsBody) -> ConstValue<'item> {
        for statement in &body.statements {
            match statement {
                Statement::Return(statement) => return self.expr_value(&statement.value),
                Statement::Assignment(statement) => {
                    if self.run_assignment_statement(statement).is_err() {
                        return ConstValue::Unknown;
                    }
                }
            }
        }
        ConstValue::Unknown
    }

    fn run_assignment_statement(&mut self, node: &AssignmentStatement) -> Result<(), ()> {
        let assigned_param = self.param(&node.assigned).ok_or(())?;
        let new_value = self.expr_value(&node.value);
        let param_value = self
            .scope_values
            .last_mut()
            .and_then(|values| values.get_mut(&assigned_param.id))
            .unwrap_or_else(|| unreachable!("param should be registered before"));
        *param_value = new_value;
        Ok(())
    }

    fn param(&self, expr: &Expr) -> Option<&'item Param> {
        match expr {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_)
            | Expr::Call(_) => None,
            Expr::Ident(ident) => match self.indexes.sources.get(&ident.id)? {
                ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Fn(_) => None,
                ItemRef::Param(param) => Some(param),
            },
        }
    }

    fn run_scoped<O>(&mut self, callback: impl FnOnce(&mut Self) -> O) -> O {
        self.enter_scope();
        let output = callback(self);
        self.exit_scope();
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ConstValue<'item> {
    TypeRef(&'item StructDefinition),
    Param(&'item Param),
    I32(i32),
    U32(u32),
    F32(HashableF32),
    Bool(bool),
    Unknown,
    RuntimeValue,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HashableF32(pub(crate) f32);

impl PartialEq for HashableF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for HashableF32 {}

impl Hash for HashableF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
