use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::fns::{CompilerImpl, FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, Statement};
use crate::compiler::values::ValueResolver;
use crate::compiler::values::types::Type;
use std::hash::{Hash, Hasher};

impl<'item> ValueResolver<'item, '_> {
    pub(crate) fn add_value(&mut self, id: u64, value: ConstValue<'item>) {
        self.scopes
            .last_mut()
            .unwrap_or_else(|| unreachable!("constant value scope should be entered"))
            .const_values
            .insert(id, value);
    }

    pub(crate) fn expr_const_value(&mut self, expr: &Expr) -> ConstValue<'item> {
        match expr {
            Expr::F32Literal(node) => Self::f32_literal_value(node),
            Expr::U32Literal(node) => Self::u32_literal_value(node),
            Expr::I32Literal(node) => Self::i32_literal_value(node),
            Expr::BoolLiteral(node) => ConstValue::Bool(node.value),
            Expr::Wildcard(_) => ConstValue::Unknown,
            Expr::Call(node) => self.call_const_value(node),
            Expr::Ident(node) => self.ident_const_value(node),
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

    fn ident_const_value(&mut self, node: &Ident) -> ConstValue<'item> {
        match self.indexes.sources.get(&node.id) {
            Some(ItemRef::Var(_)) => ConstValue::RuntimeValue,
            Some(ItemRef::Const(child)) => self.expr_const_value(&child.value),
            Some(ItemRef::Struct(child)) => ConstValue::TypeRef(child),
            Some(ItemRef::Param(child)) => {
                let value = self.const_value(child.id);
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

    fn call_const_value(&mut self, node: &Call) -> ConstValue<'item> {
        match self.indexes.sources.get(&node.id) {
            Some(ItemRef::Fn(source)) => self.fn_call_const_value(node, source),
            Some(ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_)) => {
                unreachable!("identifier should not refer to a value")
            }
            None => ConstValue::Unknown,
        }
    }

    fn fn_call_const_value(&mut self, node: &Call, source: &FnDefinition) -> ConstValue<'item> {
        debug_assert_eq!(node.args.len(), source.params.params.len());
        if ItemRef::Fn(source).is_param_constness_ignored() {
            return self.fn_compilerimpl_const_value(node, source);
        }
        let param_args = node
            .args
            .iter()
            .zip(&source.params.params)
            .map(|(arg, param)| (param.id, self.expr_const_value(arg)))
            .collect::<Vec<_>>();
        self.run_scoped(|self_| {
            for (param_id, arg_value) in param_args {
                match arg_value {
                    ConstValue::TypeRef(_)
                    | ConstValue::Param(_)
                    | ConstValue::WildcardType(_)
                    | ConstValue::I32(_)
                    | ConstValue::U32(_)
                    | ConstValue::F32(_)
                    | ConstValue::Bool(_) => self_.add_value(param_id, arg_value),
                    ConstValue::Unknown | ConstValue::RuntimeValue => return arg_value,
                }
            }
            self_.fn_const_value(node, source)
        })
    }

    fn fn_const_value(&mut self, call: &Call, source: &FnDefinition) -> ConstValue<'item> {
        if source.const_keyword_span.is_none() {
            return ConstValue::RuntimeValue;
        }
        match &source.body {
            FnBody::Compilerimpl(_) => self.fn_compilerimpl_const_value(call, source),
            FnBody::Statements(body) => self.fn_body_const_value(body),
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn fn_compilerimpl_const_value(
        &mut self,
        call: &Call,
        source: &FnDefinition,
    ) -> ConstValue<'item> {
        match source.compilerimpl() {
            Some(CompilerImpl::Add) => {
                let left = self.const_value(source.params.params[0].id);
                let right = self.const_value(source.params.params[1].id);
                match (left, right) {
                    (ConstValue::I32(left), ConstValue::I32(right)) => {
                        ConstValue::I32(left.wrapping_add(right))
                    }
                    _ => unreachable!("not implemented `{}` constant GPU function", source.name),
                }
            }
            Some(CompilerImpl::Typeof) => match self.expr_type(&call.args[0]) {
                Type::Struct(type_) => ConstValue::TypeRef(type_),
                Type::Param(param) => ConstValue::Param(param),
                Type::Wildcard(param) => ConstValue::WildcardType(param),
                Type::NoReturn | Type::Unknown => ConstValue::Unknown,
            },
            Some(CompilerImpl::Sizeof) => match self.const_value(source.params.params[0].id) {
                ConstValue::TypeRef(type_) => ConstValue::U32(type_.size()),
                _ => unreachable!("not implemented `{}` constant GPU function", source.name),
            },
            None => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    fn fn_body_const_value(&mut self, body: &FnStatementsBody) -> ConstValue<'item> {
        for statement in &body.statements {
            match statement {
                Statement::Return(statement) => {
                    return self.expr_const_value(&statement.value);
                }
                Statement::Assignment(statement) => {
                    if self.run_const_assignment_statement(statement).is_err() {
                        return ConstValue::Unknown;
                    }
                }
            }
        }
        ConstValue::Unknown
    }

    fn run_const_assignment_statement(&mut self, node: &AssignmentStatement) -> Result<(), ()> {
        let assigned_param = self.param(&node.assigned).ok_or(())?;
        let new_value = self.expr_const_value(&node.value);
        let param_value = self
            .scopes
            .last_mut()
            .and_then(|scope| scope.const_values.get_mut(&assigned_param.id))
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
            | Expr::Wildcard(_)
            | Expr::Call(_) => None,
            Expr::Ident(ident) => match self.indexes.sources.get(&ident.id)? {
                ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Fn(_) => None,
                ItemRef::Param(param) => Some(param),
            },
        }
    }

    fn const_value(&self, id: u64) -> ConstValue<'item> {
        self.scopes
            .last()
            .and_then(|scope| scope.const_values.get(&id))
            .cloned()
            .unwrap_or(ConstValue::RuntimeValue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ConstValue<'item> {
    TypeRef(&'item StructDefinition),
    Param(&'item Param),
    WildcardType(&'item Param),
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
