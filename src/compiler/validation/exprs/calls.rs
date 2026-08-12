use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::COMMENT_PREFIX;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::IntrinsicType;
use crate::compiler::validation::{ParamConstness, ValidateState, exprs, logs};
use crate::compiler::{queries, types};
use crate::utils::parsing::span::SpanProps;
use crate::utils::validation::ValidateError;
use itertools::Itertools;

pub(crate) fn validate_call(
    call: &Call,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let source = state.inner.sources.get(&call.id).copied();
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
                is_error_detected |= exprs::validate_expr(&arg.value, state).is_err(); // no-fn-check (recursivity)
            });
        });
    }
    if is_error_detected {
        return Err(ValidateError);
    }
    validate_contradicted_fact(call, state)?;
    let displayed_key = key_rendering::call_key(call, state.inner)?;
    let source =
        exprs::validate_source(source, call, call.span, &call.key(), &displayed_key, state)?;
    for (arg, param) in call.args.iter().zip(&source.params().params) {
        // Error is ignored because it is isolated from other errors
        _ = validate_arg_name(arg, param, state);
    }
    if let Some(const_mark_span) = state.const_mark_span {
        exprs::validate_const_value(
            source,
            call.span,
            const_mark_span,
            state.param_constness,
            state,
        )?;
    }
    if queries::calls::is_const_infinite_f32(call, state.inner) {
        state.add_log(logs::exprs::f32_const_out_of_bounds(call.span, state));
        return Err(ValidateError);
    }
    validate_mul_add_candidate(call, state);
    Ok(())
}

fn validate_contradicted_fact(
    call: &Call,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if let Some(fact_subject_spans) = state.inner.contradicted_type_fact_subject_spans(call.id) {
        let fact_subjects = fact_subject_spans
            .iter()
            .map(|span| state.context.slice(*span));
        state.add_log(logs::exprs::contradicted_type_fact(
            fact_subjects,
            call.span,
            state,
        ));
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn validate_arg_name(
    arg: &Arg,
    param: &Param,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let Some(name) = &arg.name else {
        return Ok(());
    };
    if name == &param.name {
        Ok(())
    } else {
        state.add_log(logs::calls::arg_name_mismatch(
            name,
            arg.name_span,
            &param.name,
            param.name_span,
            state,
        ));
        Err(ValidateError)
    }
}

fn validate_mul_add_candidate(call: &Call, state: &mut ValidateState<'_, '_>) {
    if !are_all_args_f32(call, state) || !is_call_intrinsic_add(call, state) {
        return;
    }
    let Some(replacement) = mul_add_replacement(call, state) else {
        return;
    };
    state.add_log(logs::exprs::mul_add_candidate(
        call.span,
        &replacement,
        state,
    ));
}

fn mul_add_replacement(call: &Call, state: &ValidateState<'_, '_>) -> Option<String> {
    let left = unparenthesized(&call.args[0].value);
    let right = unparenthesized(&call.args[1].value);
    let (mul_call, addend) = match (left, right) {
        (Expr::Call(mul_call), addend) if is_call_intrinsic_mul(mul_call, state) => {
            (mul_call, addend)
        }
        (addend, Expr::Call(mul_call)) if is_call_intrinsic_mul(mul_call, state) => {
            (mul_call, addend)
        }
        _ => return None,
    };
    let left = format_arg(state.context.slice(mul_call.args[0].value.span()));
    let right = format_arg(state.context.slice(mul_call.args[1].value.span()));
    let addend = format_arg(state.context.slice(addend.span()));
    Some(format!("mul_add({left}, {right}, {addend})"))
}

fn unparenthesized(expr: &Expr) -> &Expr {
    match expr {
        Expr::Parenthesized(parenthesized) => unparenthesized(&parenthesized.value),
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_)
        | Expr::Ident(_) => expr,
    }
}

fn format_arg(source: &str) -> String {
    source
        .lines()
        .map(format_arg_line)
        .filter(|line| !line.is_empty())
        .join(" ")
}

fn format_arg_line(line: &str) -> &str {
    line.split_once(COMMENT_PREFIX)
        .map_or(line, |(code, _)| code)
        .trim()
}

fn is_call_intrinsic_add(call: &Call, state: &ValidateState<'_, '_>) -> bool {
    queries::calls::is_intrinsic(
        call,
        IntrinsicFn::Binary(BinaryIntrinsicFn::Add),
        state.inner,
    )
}

fn is_call_intrinsic_mul(call: &Call, state: &ValidateState<'_, '_>) -> bool {
    queries::calls::is_intrinsic(
        call,
        IntrinsicFn::Binary(BinaryIntrinsicFn::Mul),
        state.inner,
    )
}

fn are_all_args_f32(call: &Call, state: &ValidateState<'_, '_>) -> bool {
    call.args.iter().all(|arg| {
        state.inner.is_intrinsic_type(
            types::expr_type(&arg.value, state.inner),
            IntrinsicType::F32,
        )
    })
}
