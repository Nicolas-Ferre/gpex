use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::{Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};
use itertools::Itertools;

pub(crate) fn check_found(
    node: impl NodeRef,
    span: Span,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if indexes.sources.contains_key(&node.id()) {
        Ok(())
    } else {
        let slice = context.slice(span);
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{slice}` value not found"),
            location: Some(context.location(span)),
            inner: if let Some(private_source) = indexes.private_sources.get(&node.id()) {
                vec![LogInner {
                    level: LogLevel::Info,
                    message: "value not qualified with `pub`".into(),
                    location: Some(context.location(private_source.name_span())),
                }]
            } else {
                indexes
                    .items
                    .iter_by_key(slice)
                    .filter(ItemNodeRef::is_public)
                    .map(|item| LogInner {
                        level: LogLevel::Info,
                        message: format!(
                            "value can be imported from `{}`",
                            context.dot_path(item.file_index())
                        ),
                        location: Some(context.location(item.name_span())),
                    })
                    .collect()
            },
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_constant(
    node: impl NodeRef,
    span: Span,
    constant_mark_span: Span,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if matches!(
        indexes.sources[&node.id()],
        ItemRef::Constant(_) | ItemRef::Struct(_)
    ) {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "expression not constant".into(),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "expression must be constant".into(),
                location: Some(context.location(constant_mark_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_char_count(span: Span, context: &mut ValidateContext<'_>) {
    let slice = context.slice(span);
    if slice.len() == 1 && slice != "_" {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{slice}` identifier is single character"),
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
            message: format!("`{slice}` identifier not in {case_labels}"),
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
            Self::Pascal => slice.char_indices().all(|(index, char)| {
                char.is_ascii_uppercase() || char.is_ascii_digit() || (index == 0 && char == '_')
            }),
        }
    }
}
