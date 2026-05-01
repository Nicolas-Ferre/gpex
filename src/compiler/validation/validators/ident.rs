use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::ValidateContext;
use crate::{Log, LogLevel};
use itertools::Itertools;

pub(crate) fn check_char_count(span: Span, context: &mut ValidateContext<'_>) {
    let slice = context.slice(span);
    if slice.len() == 1 && slice != "_" {
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{slice}` identifier is single character"),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

pub(crate) fn check_case(span: Span, expected_cases: &[Case], context: &mut ValidateContext<'_>) {
    let slice = context.slice(span);
    if !expected_cases.iter().any(|case| case.is_valid(slice)) {
        let case_labels = expected_cases.iter().map(|case| case.labels()).join(" or ");
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{slice}` identifier not in {case_labels}"),
            location: Some(context.location(span)),
            inner: vec![],
        });
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
