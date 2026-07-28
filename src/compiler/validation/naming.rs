use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::vars::ConstDefinition;
use crate::compiler::parsing::symbols::KEYWORDS;
use crate::compiler::state::IntrinsicType;
use crate::compiler::types;
use crate::compiler::types::Type;
use crate::compiler::validation::{ValidateState, logs};
use crate::utils::casing;
use crate::utils::parsing::span::{Span, SpanProps};
use convert_case::Case;

pub(super) const IMPORT_ALLOWED_CASES: &[Case<'static>] = &[Case::Snake];
pub(super) const VAR_ALLOWED_CASES: &[Case<'static>] = &[Case::Snake];

pub(super) fn validate_name(
    span: Span,
    expected_cases: &[Case<'_>],
    state: &mut ValidateState<'_, '_>,
) {
    validate_char_count(span, state);
    validate_case(span, expected_cases, state);
}

pub(super) fn validate_char_count(span: Span, state: &mut ValidateState<'_, '_>) {
    let slice = state.context.slice(span);
    if slice.len() == 1 && slice != "_" {
        state.add_log(logs::idents::single_char(slice, span, state));
    }
}

pub(super) fn validate_case(
    span: Span,
    expected_cases: &[Case<'_>],
    state: &mut ValidateState<'_, '_>,
) {
    let slice = state.context.slice(span);
    let mut replacements = Vec::with_capacity(expected_cases.len());
    for case in expected_cases {
        let mut replacement = casing::convert(slice, *case);
        if casing::is_valid(slice, *case, &replacement) {
            return;
        }
        make_keyword_safe(&mut replacement);
        replacements.push(replacement);
    }
    let case_labels = expected_cases.iter().map(|case| case_label(*case));
    state.add_log(logs::idents::invalid_case(
        slice,
        span,
        case_labels,
        replacements.into_iter(),
        state,
    ));
}

pub(super) fn const_cases(
    const_: &ConstDefinition,
    state: &ValidateState<'_, '_>,
) -> &'static [Case<'static>] {
    let type_ = types::expr_type(&const_.value, state.inner);
    let is_typeref = state.inner.is_intrinsic_type(type_, IntrinsicType::Typeref);
    let may_be_typeref = type_.struct_ref().is_none() || is_typeref;
    if may_be_typeref {
        &[Case::UpperSnake, Case::Pascal]
    } else {
        &[Case::UpperSnake]
    }
}

pub(super) fn fn_cases(
    fn_: &FnDefinition,
    state: &ValidateState<'_, '_>,
) -> &'static [Case<'static>] {
    let type_ = types::fn_type(fn_, state.inner);
    let is_typeref = match type_ {
        Type::Struct(_) => state.inner.is_intrinsic_type(type_, IntrinsicType::Typeref),
        Type::Param(_) | Type::Wildcard(_) | Type::NoReturn => false,
        Type::Unknown => unreachable!("return type should be validated before"),
    };
    if is_typeref && fn_.const_keyword_span.is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}

pub(super) fn param_cases<'item>(
    param: &'item Param,
    state: &ValidateState<'_, 'item>,
) -> &'static [Case<'static>] {
    let type_ = types::param_type(param, state.inner);
    let is_typeref = state.inner.is_intrinsic_type(type_, IntrinsicType::Typeref);
    if is_typeref && param.const_mark_span().is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}

pub(super) fn make_keyword_safe(name: &mut String) {
    if KEYWORDS.contains(&name.as_str()) {
        name.push('_');
    }
}

#[allow(clippy::wildcard_enum_match_arm)] // opt-in is preferred
fn case_label(case: Case<'_>) -> &'static str {
    match case {
        Case::Snake => "snake_case",
        Case::UpperSnake => "SCREAMING_SNAKE_CASE",
        Case::Pascal => "PascalCase",
        _ => unreachable!("unsupported case: {case:?}"),
    }
}
