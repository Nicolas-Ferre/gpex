use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::state::State;
use crate::compiler::transpilation::{MAIN_BUFFER_NAME, SpecializedFn, compilerimpl};
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::{consts, types};
use crate::utils::{endianness, formatting};
use std::fmt::Write;

pub(super) fn transpile_expr(node: &Expr, state: &mut State<'_>) {
    let value = consts::expr_const_value(node, state);
    if value == ConstValue::RuntimeValue {
        match node {
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

pub(super) fn transpile_var_ref(node: &VarDefinition, state: &mut State<'_>) {
    state.shader += MAIN_BUFFER_NAME;
    _ = write!(state.shader, ".v{}", node.id);
}

pub(super) fn transpile_call(node: &Call, state: &mut State<'_>) {
    match state.sources[&node.id] {
        ItemRef::Fn(child) => match &child.body {
            FnBody::Compilerimpl(_) => compilerimpl::transpile_call(node, child, state),
            FnBody::Statements(body) => transpile_custom_fn_call(node, child, body, state),
        },
        ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
            unreachable!("function calls cannot reference values")
        }
    }
}

fn transpile_custom_fn_call<'item>(
    node: &Call,
    child: &'item FnDefinition,
    body: &'item FnStatementsBody,
    state: &mut State<'item>,
) {
    let specialized_fn_id = register_specialized_fn(node, child, body, state);
    _ = write!(state.shader, "_{}_{specialized_fn_id}", child.id);
    state.shader += "(";
    for (arg, param) in node.args.iter().zip(&child.params.params) {
        if param.const_mark_span().is_none() {
            transpile_expr(&arg.value, state);
            state.shader += ", ";
        }
    }
    state.shader += ")";
}

fn transpile_ident(node: &Ident, state: &mut State<'_>) {
    match state.sources[&node.id] {
        ItemRef::Var(node) => transpile_var_ref(node, state),
        ItemRef::Param(node) => transpile_param_ref(node, state),
        ItemRef::Fn(_) => unreachable!("identifiers cannot reference functions"),
        ItemRef::Const(_) | ItemRef::Struct(_) => {
            unreachable!("constant item should be transpiled in `Expression::transpile`")
        }
    }
}

fn transpile_const_value(node: &ConstValue<'_>, state: &mut State<'_>) {
    match node {
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

fn transpile_param_ref(node: &Param, state: &mut State<'_>) {
    let id = node.id;
    _ = write!(state.shader, "_{id}");
}

fn transpile_struct_ref(node: &StructDefinition, state: &mut State<'_>) {
    let [id_part1, id_part2] = endianness::to_portable_u32x2(node.id);
    _ = write!(state.shader, "vec2<u32>({id_part1}, {id_part2})");
}

fn register_specialized_fn<'item>(
    node: &Call,
    child: &'item FnDefinition,
    body: &'item FnStatementsBody,
    state: &mut State<'item>,
) -> usize {
    let specialized_fn_id = state.specialized_fns.len();
    let specialized_fn = SpecializedFn {
        fn_: child,
        const_param_values: fn_const_param_values(node, child, state),
        wildcard_param_types: fn_param_wildcard_types(node, child, state),
        fn_body: body,
    };
    *state
        .specialized_fns
        .entry(specialized_fn)
        .or_insert(specialized_fn_id)
}

fn fn_const_param_values<'item>(
    node: &Call,
    child: &FnDefinition,
    state: &mut State<'item>,
) -> Vec<ConstValue<'item>> {
    node.args
        .iter()
        .zip(&child.params.params)
        .filter(|(_, param)| param.const_mark_span().is_some())
        .map(|(arg, _)| consts::expr_const_value(&arg.value, state))
        .collect::<Vec<_>>()
}

fn fn_param_wildcard_types<'item>(
    node: &Call,
    child: &FnDefinition,
    state: &mut State<'item>,
) -> Vec<&'item StructDefinition> {
    node.args
        .iter()
        .zip(&child.params.params)
        .filter(|(_, param)| matches!(param.type_, Expr::Wildcard(_)))
        .map(|(arg, _)| {
            types::expr_type(&arg.value, state)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("argument type should be validated before"))
        })
        .collect::<Vec<_>>()
}
