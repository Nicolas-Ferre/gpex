use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{BinaryCompilerImplFn, CompilerImplFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::CompilerImplType;
use crate::compiler::validation::{ParamConstness, ValidateState, exprs, logs};
use crate::compiler::{consts, types};
use crate::utils::validation::ValidateError;

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
    if consts::is_const_infinite_f32(call, state.inner) {
        state.add_log(logs::exprs::f32_const_out_of_bounds(call.span, state));
        return Err(ValidateError);
    }
    validate_mul_add_candidate(call, source, state);
    Ok(())
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

fn validate_mul_add_candidate<'item>(
    call: &Call,
    source: ItemRef<'item>,
    state: &mut ValidateState<'_, 'item>,
) {
    let ItemRef::Fn(source) = source else {
        unreachable!("calls can only be functions")
    };
    let are_all_args_f32 = call.args.iter().all(|arg| {
        state.inner.is_compilerimpl_type(
            types::expr_type(&arg.value, state.inner),
            CompilerImplType::F32,
        )
    });
    if !are_all_args_f32
        || source.compilerimpl() != Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Add))
        || !call
            .args
            .iter()
            .any(|arg| is_expr_compilerimpl_mul(&arg.value, state))
    {
        return;
    }
    state.add_log(logs::exprs::mul_add_candidate(call.span, state));
}

fn is_expr_compilerimpl_mul(expr: &Expr, state: &ValidateState<'_, '_>) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    matches!(
        state.inner.sources.get(&call.id),
        Some(ItemRef::Fn(source))
            if source.compilerimpl()
                == Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Mul))
    )
}
