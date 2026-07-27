use crate::compiler::parsing::exprs::BINARY_FN_NAMES;
use crate::compiler::parsing::exprs::calls::UNARY_FN_NAMES;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::vars::ConstDefinition;
use crate::compiler::state::IntrinsicType;
use crate::compiler::types;
use crate::compiler::types::Type;
use crate::compiler::validation::{ValidateState, logs};
use crate::utils::parsing::span::{Span, SpanProps};
use convert_case::{Boundary, Case as ConvertCase, Converter};

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
        let replacements = expected_cases.iter().map(|case| case.convert(slice));
        state.add_log(logs::idents::invalid_case(
            slice,
            span,
            case_labels,
            replacements,
            state,
        ));
    }
}

pub(super) fn const_cases(
    const_: &ConstDefinition,
    state: &ValidateState<'_, '_>,
) -> &'static [Case] {
    let type_ = types::expr_type(&const_.value, state.inner);
    let is_typeref = state.inner.is_intrinsic_type(type_, IntrinsicType::Typeref);
    let may_be_typeref = type_.struct_ref().is_none() || is_typeref;
    if may_be_typeref {
        &[Case::ScreamingSnake, Case::Pascal]
    } else {
        &[Case::ScreamingSnake]
    }
}

pub(super) fn fn_cases(fn_: &FnDefinition, state: &ValidateState<'_, '_>) -> &'static [Case] {
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
) -> &'static [Case] {
    let type_ = types::param_type(param, state.inner);
    let is_typeref = state.inner.is_intrinsic_type(type_, IntrinsicType::Typeref);
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
        BINARY_FN_NAMES.contains(&slice)
            || UNARY_FN_NAMES.contains(&slice)
            || self.convert(slice) == slice
    }

    fn convert(self, slice: &str) -> String {
        let name = slice.trim_start_matches('_');
        let converter = Converter::new().remove_boundaries(&Boundary::digits());
        let converted_name = match self {
            Self::Snake => converter.to_case(ConvertCase::Snake).convert(name),
            Self::ScreamingSnake => converter.to_case(ConvertCase::UpperSnake).convert(name),
            Self::Pascal => converter.to_case(ConvertCase::Pascal).convert(name),
        };
        let has_leading_underscore = slice.len() - name.len() > 0;
        let underscore_prefixes = "_".repeat(has_leading_underscore.into());
        format!("{underscore_prefixes}{converted_name}")
    }
}
