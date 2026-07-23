use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::transpilation::{
    MAIN_BUFFER_NAME, SpecializedFn, TranspileState, compilerimpl,
};
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::{consts, types};
use crate::utils::{endianness, formatting};
use std::fmt::Write;

pub(super) fn transpile_expr(expr: &Expr, state: &mut TranspileState<'_, '_>) {
    let value = consts::expr_value(expr, state.inner);
    if value == ConstValue::RuntimeValue {
        match expr {
            Expr::Call(child) => transpile_call(child, state),
            Expr::Ident(child) => transpile_ident(child, state),
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_)
            | Expr::Wildcard(_) => unreachable!("expression should be validated before"),
        }
    } else {
        transpile_const_value(&value, state);
    }
}

pub(super) fn transpile_var_ref(var: &VarDefinition, state: &mut TranspileState<'_, '_>) {
    state.shader += MAIN_BUFFER_NAME;
    _ = write!(state.shader, ".v{}", var.id);
}

pub(super) fn transpile_call(call: &Call, state: &mut TranspileState<'_, '_>) {
    match state.inner.sources[&call.id] {
        ItemRef::Fn(child) => match &child.body {
            FnBody::Compilerimpl(_) => compilerimpl::transpile_call(call, child, state),
            FnBody::Statements(body) => transpile_custom_fn_call(call, child, body, state),
        },
        ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
            unreachable!("function calls cannot reference values")
        }
    }
}

fn transpile_custom_fn_call<'item>(
    call: &Call,
    child: &'item FnDefinition,
    body: &'item FnStatementsBody,
    state: &mut TranspileState<'_, 'item>,
) {
    let specialized_fn_id = register_specialized_fn(call, child, body, state);
    _ = write!(state.shader, "_{}_{specialized_fn_id}", child.id);
    state.shader += "(";
    for (arg, param) in call.args.iter().zip(&child.params.params) {
        if param.const_mark_span().is_none() {
            transpile_expr(&arg.value, state);
            state.shader += ", ";
        }
    }
    state.shader += ")";
}

fn transpile_ident(ident: &Ident, state: &mut TranspileState<'_, '_>) {
    match state.inner.sources[&ident.id] {
        ItemRef::Var(var) => transpile_var_ref(var, state),
        ItemRef::Param(param) => transpile_param_ref(param, state),
        ItemRef::Fn(_) => unreachable!("identifiers cannot reference functions"),
        ItemRef::Const(_) | ItemRef::Struct(_) => {
            unreachable!("constant item should be transpiled in `Expression::transpile`")
        }
    }
}

fn transpile_const_value(value: &ConstValue<'_>, state: &mut TranspileState<'_, '_>) {
    match value {
        ConstValue::TypeRef(value) => transpile_struct_ref(value, state),
        ConstValue::I32(value) => _ = write!(state.shader, "i32({value})"),
        ConstValue::U32(value) => _ = write!(state.shader, "u32({value})"),
        ConstValue::F32(value) => {
            _ = write!(state.shader, "f32({})", formatting::f32_to_string(value.0));
        }
        ConstValue::Bool(value) => _ = write!(state.shader, "u32({})", u32::from(*value)),
        ConstValue::Param(_)
        | ConstValue::WildcardType(_)
        | ConstValue::Unknown
        | ConstValue::RuntimeValue => {
            unreachable!("non-constant cannot be transpiled")
        }
    }
}

fn transpile_param_ref(param: &Param, state: &mut TranspileState<'_, '_>) {
    let id = param.id;
    _ = write!(state.shader, "_{id}");
}

fn transpile_struct_ref(struct_: &StructDefinition, state: &mut TranspileState<'_, '_>) {
    let [id_part1, id_part2] = endianness::to_portable_u32x2(struct_.id);
    _ = write!(state.shader, "vec2<u32>({id_part1}, {id_part2})");
}

fn register_specialized_fn<'item>(
    call: &Call,
    child: &'item FnDefinition,
    body: &'item FnStatementsBody,
    state: &mut TranspileState<'_, 'item>,
) -> usize {
    let specialized_fn_id = state.specialized_fns.len();
    let specialized_fn = SpecializedFn {
        fn_: child,
        const_param_values: fn_const_param_values(call, child, state),
        wildcard_param_types: fn_param_wildcard_types(call, child, state),
        fn_body: body,
    };
    *state
        .specialized_fns
        .entry(specialized_fn)
        .or_insert(specialized_fn_id)
}

fn fn_const_param_values<'item>(
    call: &Call,
    child: &FnDefinition,
    state: &TranspileState<'_, 'item>,
) -> Vec<ConstValue<'item>> {
    call.args
        .iter()
        .zip(&child.params.params)
        .filter(|(_, param)| param.const_mark_span().is_some())
        .map(|(arg, _)| consts::expr_value(&arg.value, state.inner))
        .collect::<Vec<_>>()
}

fn fn_param_wildcard_types<'item>(
    call: &Call,
    child: &FnDefinition,
    state: &TranspileState<'_, 'item>,
) -> Vec<&'item StructDefinition> {
    call.args
        .iter()
        .zip(&child.params.params)
        .filter(|(_, param)| matches!(param.type_, Expr::Wildcard(_)))
        .map(|(arg, _)| {
            types::expr_type(&arg.value, state.inner)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("argument type should be validated before"))
        })
        .collect::<Vec<_>>()
}
