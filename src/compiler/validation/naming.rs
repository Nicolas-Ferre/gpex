use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::vars::ConstDefinition;
use crate::compiler::state::CompilerImplType;
use crate::compiler::types;
use crate::compiler::types::Type;
use crate::compiler::validation::{ValidateState, logs};
use crate::utils::parsing::span::{Span, SpanProps};

pub(super) const IMPORT_ALLOWED_CASES: &[Case] = &[Case::Snake];
pub(super) const VAR_ALLOWED_CASES: &[Case] = &[Case::Snake];

pub(super) fn validate_name(
    span: Span,
    expected_cases: &[Case],
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
    expected_cases: &[Case],
    state: &mut ValidateState<'_, '_>,
) {
    let slice = state.context.slice(span);
    if !expected_cases.iter().any(|case| case.is_valid(slice)) {
        let case_labels = expected_cases.iter().map(|case| case.labels());
        state.add_log(logs::idents::invalid_case(slice, span, case_labels, state));
    }
}

pub(super) fn const_allowed_cases(
    const_: &ConstDefinition,
    state: &ValidateState<'_, '_>,
) -> &'static [Case] {
    let type_ = types::expr_type(&const_.value, state.inner);
    let may_be_typeref = type_.struct_ref().is_none()
        || state
            .inner
            .is_compilerimpl_type(type_, CompilerImplType::Typeref);
    if may_be_typeref {
        &[Case::ScreamingSnake, Case::Pascal]
    } else {
        &[Case::ScreamingSnake]
    }
}

pub(super) fn fn_allowed_cases(
    fn_: &FnDefinition,
    state: &ValidateState<'_, '_>,
) -> &'static [Case] {
    let type_ = types::fn_type(fn_, state.inner);
    let may_return_typeref = match type_ {
        Type::Struct(_) => state
            .inner
            .is_compilerimpl_type(type_, CompilerImplType::Typeref),
        Type::Param(_) | Type::Wildcard(_) | Type::NoReturn => false,
        Type::Unknown => unreachable!("return type should be validated before"),
    };
    if may_return_typeref && fn_.const_keyword_span.is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}

pub(super) fn param_allowed_cases<'item>(
    param: &'item Param,
    state: &ValidateState<'_, 'item>,
) -> &'static [Case] {
    let type_ = types::param_type(param, state.inner);
    let is_typeref = state
        .inner
        .is_compilerimpl_type(type_, CompilerImplType::Typeref);
    if is_typeref && param.const_mark_span().is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Case {
    Snake,
    ScreamingSnake,
    Pascal,
}

impl Case {
    fn labels(self) -> &'static str {
        match self {
            Self::Snake => "snake_case",
            Self::ScreamingSnake => "SCREAMING_SNAKE_CASE",
            Self::Pascal => "PascalCase",
        }
    }

    fn is_valid(self, slice: &str) -> bool {
        match self {
            Self::Snake => slice
                .chars()
                .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '_'),
            Self::ScreamingSnake => slice
                .chars()
                .all(|char| char.is_ascii_uppercase() || char.is_ascii_digit() || char == '_'),
            Self::Pascal => {
                let first_uppercase_index = usize::from(slice.starts_with('_'));
                slice.char_indices().all(|(index, char)| {
                    (index != first_uppercase_index || char.is_ascii_uppercase())
                        && (char.is_ascii_alphanumeric() || (index == 0 && char == '_'))
                })
            }
        }
    }
}
