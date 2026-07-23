use crate::compiler::validation::{ValidateState, logs};
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn check_char_count(span: Span, state: &mut ValidateState<'_, '_>) {
    let slice = state.context.slice(span);
    if slice.len() == 1 && slice != "_" {
        state.add_log(logs::idents::single_char(slice, span, state));
    }
}

pub(crate) fn check_case(span: Span, expected_cases: &[Case], state: &mut ValidateState<'_, '_>) {
    let slice = state.context.slice(span);
    if !expected_cases.iter().any(|case| case.is_valid(slice)) {
        let case_labels = expected_cases.iter().map(|case| case.labels());
        state.add_log(logs::idents::invalid_case(slice, span, case_labels, state));
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Case {
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
