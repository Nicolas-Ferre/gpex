pub(super) mod calls;

use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::types::Type;
use crate::compiler::validation::{ParamConstness, ValidateState, logs};
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

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
        Expr::Call(child) => validate_has_return_type(child, child.span, state)
            .and_then(|()| calls::validate_call(child, state)),
        Expr::Ident(child) => validate_ident(child, state),
    }
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

pub(super) fn validate_no_return_type(
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

fn validate_has_return_type(
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
