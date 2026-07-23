use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{BinaryCompilerImplFn, CompilerImplFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::refs;
use crate::compiler::validation::{logs, ParamConstness, ValidateState};
use crate::compiler::values::types::Type;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

pub(crate) fn check_types(
    actual_span: Span,
    actual_type: Type<'_>,
    expected_span: Option<Span>,
    expected_type: Type<'_>,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if !actual_type.is_comparable() || !expected_type.is_comparable() {
        Err(ValidateError)
    } else if actual_type == expected_type {
        Ok(())
    } else {
        let actual_type_name = actual_type.name()?;
        let expected_type_name = expected_type.name()?;
        state.add_log(logs::exprs::invalid_type(
            &actual_type_name,
            actual_span,
            &expected_type_name,
            expected_span,
            state,
        ));
        Err(ValidateError)
    }
}

pub(crate) fn check_const_value(
    source: ItemRef<'_>,
    span: Span,
    const_mark_span: Span,
    param_constness: ParamConstness,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if is_item_const(source, param_constness) {
        Ok(())
    } else {
        state.add_log(logs::exprs::non_const(span, const_mark_span, state));
        Err(ValidateError)
    }
}

pub(crate) fn check_f32_const_bounds(
    is_out_of_bounds: bool,
    span: Span,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if is_out_of_bounds {
        state.add_log(logs::exprs::f32_const_out_of_bounds(span, state));
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_mul_add_candidate(
    source: ItemRef<'_>,
    call: &Call,
    are_all_args_f32: bool,
    state: &mut ValidateState<'_, '_>,
) {
    let ItemRef::Fn(source) = source else {
        unreachable!("calls can only be functions")
    };
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

pub(crate) fn check_arg_name(
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

pub(crate) fn check_no_return_type(
    node: impl NodeRef,
    span: Span,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if let Some(ItemRef::Fn(fn_)) = state.inner.sources.get(&node.id()).copied()
        && fn_.return_type.is_none()
    {
        let fn_key = key_rendering::fn_key(fn_, state.inner)?;
        state.add_log(logs::calls::called_fn_without_return_type(
            &fn_key,
            span,
            fn_.signature_span_with_return,
            state,
        ));
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_has_return_type(
    node: impl NodeRef,
    span: Span,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if let Some(ItemRef::Fn(fn_)) = state.inner.sources.get(&node.id()).copied()
        && fn_.return_type.is_some()
    {
        let fn_key = key_rendering::fn_key(fn_, state.inner)?;
        state.add_log(logs::calls::repeated_fn_with_return_type(
            &fn_key,
            span,
            fn_.signature_span_with_return,
            state,
        ));
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_ref(expr: &Expr, state: &mut ValidateState<'_, '_>) {
    if refs::is_expr_ref(expr, state.inner) == Some(false) {
        state.add_log(logs::exprs::not_ref(expr.span(), state));
    }
}

pub(crate) fn report_invalid_wildcard_location(
    span: Span,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    state.add_log(logs::exprs::invalid_wildcard(span, state));
    Err(ValidateError)
}

fn is_item_const(item: ItemRef<'_>, param_constness: ParamConstness) -> bool {
    match item {
        ItemRef::Var(_) => false,
        ItemRef::Const(_) | ItemRef::Struct(_) => true,
        ItemRef::Fn(fn_) => fn_.const_keyword_span.is_some(),
        ItemRef::Param(param) => match param_constness {
            ParamConstness::ExplicitOnly => param.const_mark_span().is_some(),
            ParamConstness::All => true,
        },
    }
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
