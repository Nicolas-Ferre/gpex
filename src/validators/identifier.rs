use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::{Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

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
    if matches!(indexes.sources[&node.id()], ItemRef::Constant(_)) {
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

pub(crate) fn check_snake_case(span: Span, context: &mut ValidateContext<'_>) {
    let slice = context.slice(span);
    if !is_snake_case(slice) {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{slice}` identifier not in snake_case"),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

pub(crate) fn check_screaming_snake_case(span: Span, context: &mut ValidateContext<'_>) {
    let slice = context.slice(span);
    if !is_screaming_snake_case(slice) {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{slice}` identifier not in SCREAMING_SNAKE_CASE"),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

fn is_snake_case(token: &str) -> bool {
    token
        .chars()
        .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '_')
}

fn is_screaming_snake_case(token: &str) -> bool {
    token
        .chars()
        .all(|char| char.is_ascii_uppercase() || char.is_ascii_digit() || char == '_')
}
