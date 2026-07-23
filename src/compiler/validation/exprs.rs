use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::{CompilerImplType, ParamConstness, State};
use crate::compiler::validation::validators;
use crate::compiler::values::{consts, types};
use crate::utils::validation::ValidateError;

pub(crate) fn validate_expr(expr: &Expr, state: &mut State<'_>) -> Result<(), ValidateError> {
    match expr {
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
    literal: &F32Literal,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    validators::literal::check_bounds(literal.value.is_some(), literal.span, "f32", state)
}

pub(crate) fn validate_i32_literal(
    literal: &I32Literal,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    validators::literal::check_bounds(literal.value.is_some(), literal.span, "i32", state)
}

pub(crate) fn validate_u32_literal(
    literal: &U32Literal,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    validators::literal::check_bounds(literal.value.is_some(), literal.span, "u32", state)
}

pub(crate) fn validate_call(call: &Call, state: &mut State<'_>) -> Result<(), ValidateError> {
    let source = state.sources.get(&call.id).copied();
    let is_constness_ignored = source.is_some_and(ItemRef::is_param_constness_ignored);
    let mut is_error_detected = false;
    for (index, arg) in call.args.iter().enumerate() {
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
    let displayed_key = key_rendering::call_key(call, state)?;
    let source =
        validators::item::check_found(source, call, call.span, &call.key(), &displayed_key, state)?;
    for (arg, param) in call.args.iter().zip(&source.params().params) {
        // Error is ignored because it is isolated from other errors
        _ = validators::expr::check_arg_name(arg, param, state);
    }
    if let Some(const_mark_span) = state.const_mark_span {
        validators::expr::check_const_value(
            source,
            call.span,
            const_mark_span,
            state.param_constness,
            state,
        )?;
    }
    let is_const_infinite_f32 = consts::is_const_infinite_f32(call, state);
    validators::expr::check_f32_const_bounds(is_const_infinite_f32, call.span, state)?;
    validate_mul_add_candidate(call, source, state);
    Ok(())
}

pub(crate) fn validate_ident(ident: &Ident, state: &mut State<'_>) -> Result<(), ValidateError> {
    let source = state.sources.get(&ident.id).copied();
    let source = validators::item::check_found(
        source,
        ident,
        ident.span,
        &ident.slice,
        &ident.slice,
        state,
    )?;
    if let Some(const_mark_span) = state.const_mark_span {
        validators::expr::check_const_value(
            source,
            ident.span,
            const_mark_span,
            state.param_constness,
            state,
        )?;
    }
    Ok(())
}

fn validate_mul_add_candidate<'item>(
    call: &Call,
    source: ItemRef<'item>,
    state: &mut State<'item>,
) {
    let are_all_args_f32 = call.args.iter().all(|arg| {
        let type_ = types::expr_type(&arg.value, state);
        state.is_compilerimpl_type(type_, CompilerImplType::F32)
    });
    validators::expr::check_mul_add_candidate(source, call, are_all_args_f32, state);
}
