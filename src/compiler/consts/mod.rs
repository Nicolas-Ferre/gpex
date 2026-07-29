mod intrinsic;

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
use crate::compiler::types;
use std::hash::{Hash, Hasher};

pub(crate) fn expr_value<'item>(expr: &Expr, state: &State<'item>) -> ConstValue<'item> {
    match expr {
        Expr::F32Literal(literal) => f32_literal_value(literal),
        Expr::U32Literal(literal) => u32_literal_value(literal),
        Expr::I32Literal(literal) => i32_literal_value(literal),
        Expr::BoolLiteral(literal) => ConstValue::Bool(literal.value),
        Expr::Wildcard(_) => ConstValue::Unknown,
        Expr::Call(call) => call_value(call, state),
        Expr::Ident(ident) => ident_value(ident, state),
        Expr::Parenthesized(parenthesized) => expr_value(&parenthesized.value, state),
    }
}

pub(crate) fn call_value<'item>(call: &Call, state: &State<'item>) -> ConstValue<'item> {
    match state.sources.get(&call.id).copied() {
        Some(ItemRef::Fn(source)) => fn_call_value(call, source, state),
        Some(ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_)) => {
            unreachable!("identifier should not refer to a value")
        }
        None => ConstValue::Unknown,
    }
}

fn i32_literal_value(literal: &I32Literal) -> ConstValue<'static> {
    if let Some(value) = literal.value {
        ConstValue::I32(value)
    } else {
        ConstValue::Unknown
    }
}

fn u32_literal_value(literal: &U32Literal) -> ConstValue<'static> {
    if let Some(value) = literal.value {
        ConstValue::U32(value)
    } else {
        ConstValue::Unknown
    }
}

fn f32_literal_value(literal: &F32Literal) -> ConstValue<'static> {
    if let Some(value) = literal.value {
        ConstValue::F32(HashableF32(value))
    } else {
        ConstValue::Unknown
    }
}

fn ident_value<'item>(ident: &Ident, state: &State<'item>) -> ConstValue<'item> {
    match state.sources.get(&ident.id).copied() {
        Some(ItemRef::Var(_)) => ConstValue::RuntimeValue,
        Some(ItemRef::Const(child)) => expr_value(&child.value, state),
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

fn fn_call_value<'item>(
    call: &Call,
    source: &'item FnDefinition,
    state: &State<'item>,
) -> ConstValue<'item> {
    debug_assert_eq!(call.args.len(), source.params.params.len());
    if ItemRef::Fn(source).is_param_constness_ignored() {
        return intrinsic::call_value(call, source, state);
    }
    let param_args = call
        .args
        .iter()
        .zip(&source.params.params)
        .map(|(arg, param)| {
            (
                param,
                expr_value(&arg.value, state),
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
        fn_value(call, source, state_)
    })
}

fn fn_value<'item>(
    call: &Call,
    source: &'item FnDefinition,
    state: &State<'item>,
) -> ConstValue<'item> {
    if source.const_keyword_span.is_none() {
        return ConstValue::RuntimeValue;
    }
    match &source.body {
        FnBody::Intrinsic(_) => intrinsic::call_value(call, source, state),
        FnBody::Statements(body) => fn_body_value(body, state),
    }
}

fn fn_body_value<'item>(body: &FnStatementsBody, state: &State<'item>) -> ConstValue<'item> {
    for statement in &body.statements {
        match statement {
            Statement::Return(return_) => {
                return expr_value(&return_.value, state);
            }
            Statement::Assignment(assignment) => {
                if run_assignment_statement(assignment, state).is_err() {
                    return ConstValue::Unknown;
                }
            }
        }
    }
    ConstValue::Unknown
}

fn run_assignment_statement(assignment: &AssignmentStatement, state: &State<'_>) -> Result<(), ()> {
    let assigned_param = param(&assignment.assigned, state).ok_or(())?;
    let new_value = expr_value(&assignment.value, state);
    let assigned_value = if matches!(new_value, ConstValue::RuntimeValue) {
        ConstValue::Unknown // runtime value in a constant assignment means the code is invalid
    } else {
        new_value
    };
    state.add_const_value(assigned_param.id, assigned_value);
    Ok(())
}

fn param<'item>(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
    match expr {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_)
        | Expr::Parenthesized(_) => None,
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
