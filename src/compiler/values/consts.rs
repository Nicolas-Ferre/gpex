use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, Statement};
use crate::compiler::state::State;
use crate::compiler::values::{compilerimpl, types};
use std::hash::{Hash, Hasher};

// TODO: in the whole project, replace "node" by a more representative name (e.g. call, fn_, var_, ...)

pub(crate) fn expr_const_value<'item>(node: &Expr, state: &mut State<'item>) -> ConstValue<'item> {
    match node {
        Expr::F32Literal(node) => f32_literal_value(node),
        Expr::U32Literal(node) => u32_literal_value(node),
        Expr::I32Literal(node) => i32_literal_value(node),
        Expr::BoolLiteral(node) => ConstValue::Bool(node.value),
        Expr::Wildcard(_) => ConstValue::Unknown,
        Expr::Call(node) => call_const_value(node, state),
        Expr::Ident(node) => ident_const_value(node, state),
    }
}

pub(crate) fn is_const_infinite_f32(node: &Call, state: &mut State<'_>) -> bool {
    matches!(
        call_const_value(node, state),
        ConstValue::F32(value) if !value.0.is_finite()
    )
}

pub(crate) fn call_const_value<'item>(node: &Call, state: &mut State<'item>) -> ConstValue<'item> {
    match state.sources.get(&node.id).copied() {
        Some(ItemRef::Fn(source)) => fn_call_const_value(node, source, state),
        Some(ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_)) => {
            unreachable!("identifier should not refer to a value")
        }
        None => ConstValue::Unknown,
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

fn ident_const_value<'item>(node: &Ident, state: &mut State<'item>) -> ConstValue<'item> {
    match state.sources.get(&node.id).copied() {
        Some(ItemRef::Var(_)) => ConstValue::RuntimeValue,
        Some(ItemRef::Const(child)) => expr_const_value(&child.value, state),
        Some(ItemRef::Struct(child)) => ConstValue::TypeRef(child),
        Some(ItemRef::Param(child)) => {
            let value = state.const_value(child.id);
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

fn fn_call_const_value<'item>(
    node: &Call,
    source: &'item FnDefinition,
    state: &mut State<'item>,
) -> ConstValue<'item> {
    debug_assert_eq!(node.args.len(), source.params.params.len());
    if ItemRef::Fn(source).is_param_constness_ignored() {
        return compilerimpl::call_const_value(node, source, state);
    }
    let param_args = node
        .args
        .iter()
        .zip(&source.params.params)
        .map(|(arg, param)| {
            (
                param,
                expr_const_value(&arg.value, state),
                types::expr_type(&arg.value, state),
            )
        })
        .collect::<Vec<_>>();
    state.in_scope(|state_| {
        for (param, arg_value, arg_type) in param_args {
            if matches!(param.type_, Expr::Wildcard(_)) {
                state_.add_wildcard_type(param.id, arg_type);
            }
            match arg_value {
                ConstValue::TypeRef(_)
                | ConstValue::Param(_)
                | ConstValue::WildcardType(_)
                | ConstValue::I32(_)
                | ConstValue::U32(_)
                | ConstValue::F32(_)
                | ConstValue::Bool(_) => state_.add_const_value(param.id, arg_value),
                ConstValue::Unknown | ConstValue::RuntimeValue => return arg_value,
            }
        }
        fn_const_value(node, source, state_)
    })
}

fn fn_const_value<'item>(
    call: &Call,
    source: &'item FnDefinition,
    state: &mut State<'item>,
) -> ConstValue<'item> {
    if source.const_keyword_span.is_none() {
        return ConstValue::RuntimeValue;
    }
    match &source.body {
        FnBody::Compilerimpl(_) => compilerimpl::call_const_value(call, source, state),
        FnBody::Statements(body) => fn_body_const_value(body, state),
    }
}

fn fn_body_const_value<'item>(
    body: &FnStatementsBody,
    state: &mut State<'item>,
) -> ConstValue<'item> {
    for statement in &body.statements {
        match statement {
            Statement::Return(statement) => {
                return expr_const_value(&statement.value, state);
            }
            Statement::Assignment(statement) => {
                if run_const_assignment_statement(statement, state).is_err() {
                    return ConstValue::Unknown;
                }
            }
        }
    }
    ConstValue::Unknown
}

fn run_const_assignment_statement(
    node: &AssignmentStatement,
    state: &mut State<'_>,
) -> Result<(), ()> {
    let assigned_param = param(&node.assigned, state).ok_or(())?;
    let new_value = expr_const_value(&node.value, state);
    let param_value = state
        .scopes
        .last_mut()
        .and_then(|scope| scope.const_values.get_mut(&assigned_param.id))
        .unwrap_or_else(|| unreachable!("param should be registered before"));
    *param_value = if matches!(new_value, ConstValue::RuntimeValue) {
        ConstValue::Unknown // runtime value in a constant assignment means the code is invalid
    } else {
        new_value
    };
    Ok(())
}

fn param<'item>(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
    match expr {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_) => None,
        Expr::Ident(ident) => match state.sources.get(&ident.id)? {
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Fn(_) => None,
            ItemRef::Param(param) => Some(param),
        },
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
