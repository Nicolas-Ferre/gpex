use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{BinaryCompilerImplFn, CompilerImplFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::CompilerImplType;
use crate::compiler::validation::{ParamConstness, ValidateState, logs};
use crate::compiler::values::types::Type;
use crate::compiler::values::{consts, types};
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

// TODO: split file

pub(crate) fn validate_expr(
    expr: &Expr,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    match expr {
        Expr::F32Literal(child) => {
            validate_literal(child.value.is_some(), child.span, "f32", state)
        }
        Expr::U32Literal(child) => {
            validate_literal(child.value.is_some(), child.span, "u32", state)
        }
        Expr::I32Literal(child) => {
            validate_literal(child.value.is_some(), child.span, "i32", state)
        }
        Expr::BoolLiteral(_) => Ok(()),
        Expr::Wildcard(span) => {
            state.add_log(logs::exprs::invalid_wildcard(*span, state));
            Err(ValidateError)
        }
        Expr::Call(child) => validate_no_return_type(child, child.span, state)
            .and_then(|()| validate_call(child, state)),
        Expr::Ident(child) => validate_ident(child, state),
    }
}

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
                is_error_detected |= validate_expr(&arg.value, state).is_err(); // no-fn-check (recursivity)
            });
        });
    }
    if is_error_detected {
        return Err(ValidateError);
    }
    let displayed_key = key_rendering::call_key(call, state.inner)?;
    let source = validate_source(source, call, call.span, &call.key(), &displayed_key, state)?;
    for (arg, param) in call.args.iter().zip(&source.params().params) {
        // Error is ignored because it is isolated from other errors
        _ = validate_arg_name(arg, param, state);
    }
    if let Some(const_mark_span) = state.const_mark_span {
        validate_const_value(
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

pub(crate) fn validate_ident(
    ident: &Ident,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let source = state.inner.sources.get(&ident.id).copied();
    let source = validate_source(source, ident, ident.span, &ident.slice, &ident.slice, state)?;
    if let Some(const_mark_span) = state.const_mark_span {
        validate_const_value(
            source,
            ident.span,
            const_mark_span,
            state.param_constness,
            state,
        )?;
    }
    Ok(())
}

pub(super) fn validate_type_match(
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

pub(super) fn validate_has_return_type(
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

fn validate_literal(
    is_value_valid: bool,
    span: Span,
    type_name: &str,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if is_value_valid {
        Ok(())
    } else {
        state.add_log(logs::exprs::literal_out_of_bounds(type_name, span, state));
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

fn validate_const_value(
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

fn validate_no_return_type(
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

fn validate_source<'item>(
    source: Option<ItemRef<'item>>,
    node: impl NodeRef,
    span: Span,
    key: &str,
    displayed_key: &str,
    state: &mut ValidateState<'_, 'item>,
) -> Result<ItemRef<'item>, ValidateError> {
    if let Some(source) = source {
        Ok(source)
    } else {
        let log = if let Some(candidates) = state.inner.candidate_sources.get(&node.id()) {
            let candidates = candidates
                .iter()
                .map(|candidate| candidate.signature_span_with_return());
            logs::items::not_found_with_similar_candidates(displayed_key, span, candidates, state)
        } else if let Some(priv_source) = state.inner.priv_sources.get(&node.id()) {
            logs::items::not_found_with_priv_candidate(
                displayed_key,
                span,
                priv_source.name_span(),
                state,
            )
        } else {
            let candidates = state
                .inner
                .items
                .iter_by_key(key)
                .filter(ItemNodeRef::is_pub)
                .map(|item| (state.context.dot_path(item.file_index()), item.name_span()));
            logs::items::not_found_with_importable_candidate(displayed_key, span, candidates, state)
        };
        state.add_log(log);
        Err(ValidateError)
    }
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
