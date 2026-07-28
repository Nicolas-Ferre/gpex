use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogLevel};
use itertools::Itertools;

pub(crate) fn single_char(ident: &str, span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: format!("`{ident}` identifier is single character"),
        location: Some(state.span_location(span)),
        inner: vec![],
    }
}

pub(crate) fn invalid_case<'label>(
    ident_name: &str,
    ident_span: Span,
    mut case_labels: impl Iterator<Item = &'label str>,
    replacements: impl Iterator<Item = String>,
    state: &ValidateState<'_, '_>,
) -> Log {
    let formatted_case_labels = case_labels.join(" or ");
    Log {
        level: LogLevel::Warning,
        msg: format!("`{ident_name}` identifier not in {formatted_case_labels}"),
        location: Some(state.span_location(ident_span)),
        inner: replacements
            .map(|replacement| super::replacement(&replacement))
            .collect(),
    }
}
