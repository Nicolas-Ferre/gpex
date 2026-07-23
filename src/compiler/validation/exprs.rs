use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::{ParamConstness, State};
use crate::compiler::validation::validators;
use crate::compiler::values::types::Type;
use crate::compiler::values::{consts, types};
use crate::utils::validation::ValidateError;

pub(crate) fn validate_expr(node: &Expr, state: &mut State<'_>) -> Result<(), ValidateError> {
    match node {
        Expr::F32Literal(child) => validate_f32_literal(child, state),
        Expr::U32Literal(child) => validate_u32_literal(child, state),
        Expr::I32Literal(child) => validate_i32_literal(child, state),
        Expr::BoolLiteral(_) => Ok(()),
        Expr::Wildcard(span) => validators::expr::report_invalid_wildcard_location(*span, state),
        Expr::Call(child) => validators::expr::check_no_return_type(child, child.span, state)
            .and_then(|()| validate_call(child, state)),
        Expr::Ident(child) => validate_ident(child, state),
    }
}

pub(crate) fn validate_f32_literal(
    node: &F32Literal,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    validators::literal::check_bounds(node.value.is_some(), node.span, "f32", state)
}

pub(crate) fn validate_i32_literal(
    node: &I32Literal,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    validators::literal::check_bounds(node.value.is_some(), node.span, "i32", state)
}

pub(crate) fn validate_u32_literal(
    node: &U32Literal,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    validators::literal::check_bounds(node.value.is_some(), node.span, "u32", state)
}

pub(crate) fn validate_call(node: &Call, state: &mut State<'_>) -> Result<(), ValidateError> {
    let source = state.sources.get(&node.id).copied();
    let is_constness_ignored = source.is_some_and(ItemRef::is_param_constness_ignored);
    let mut is_error_detected = false;
    for (index, arg) in node.args.iter().enumerate() {
        let param = source.map(|source| &source.params().params[index]);
        let param_const_mark_span = param.and_then(Param::const_mark_span);
        let const_mark_span = if is_constness_ignored {
            None
        } else {
            param_const_mark_span.or(state.const_mark_span)
        };
        let param_constness = if param_const_mark_span.is_some() {
            ParamConstness::ExplicitOnly
        } else {
            state.param_constness
        };
        state.with_param_constness(param_constness, |state| {
            state.with_const_mark_span(const_mark_span, |state| {
                is_error_detected |= validate_expr(&arg.value, state).is_err(); // no-fn-check (recursivity)
            });
        });
    }
    if is_error_detected {
        return Err(ValidateError);
    }
    let displayed_key = key_rendering::call_key(node, state)?;
    let source =
        validators::item::check_found(source, node, node.span, &node.key(), &displayed_key, state)?;
    for (arg, param) in node.args.iter().zip(&source.params().params) {
        // Error is ignored because it is isolated from other errors
        _ = validators::expr::check_arg_name(arg, param, state);
    }
    if let Some(const_mark_span) = state.const_mark_span {
        validators::expr::check_const_value(
            source,
            node.span,
            const_mark_span,
            state.param_constness,
            state,
        )?;
    }
    let is_const_infinite_f32 = consts::is_const_infinite_f32(node, state);
    validators::expr::check_f32_const_bounds(is_const_infinite_f32, node.span, state)?;
    validate_mul_add_candidate(node, source, state);
    Ok(())
}

pub(crate) fn validate_ident(node: &Ident, state: &mut State<'_>) -> Result<(), ValidateError> {
    let source = state.sources.get(&node.id).copied();
    let source =
        validators::item::check_found(source, node, node.span, &node.slice, &node.slice, state)?;
    if let Some(const_mark_span) = state.const_mark_span {
        validators::expr::check_const_value(
            source,
            node.span,
            const_mark_span,
            state.param_constness,
            state,
        )?;
    }
    Ok(())
}

fn validate_mul_add_candidate<'item>(
    node: &Call,
    source: ItemRef<'item>,
    state: &mut State<'item>,
) {
    let f32_type = state.search_prelude_type("f32");
    let are_all_args_f32 = node
        .args
        .iter()
        .all(|arg| types::expr_type(&arg.value, state) == Type::Struct(f32_type));
    validators::expr::check_mul_add_candidate(source, node, are_all_args_f32, state);
}
