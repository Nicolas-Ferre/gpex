use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_found(
    node: impl NodeRef,
    span: &Span,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if indexes.sources.contains_key(&node.id()) {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{}` value not found", span.slice),
            location: Some(context.location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_constant(
    node: impl NodeRef,
    span: &Span,
    constant_mark_span: &Span,
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

pub(crate) fn check_char_count(span: &Span, context: &mut ValidateContext<'_>) {
    if span.slice.len() == 1 && span.slice != "_" {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` identifier is single character", span.slice),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

pub(crate) fn check_snake_case(span: &Span, context: &mut ValidateContext<'_>) {
    if !is_snake_case(&span.slice) {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` identifier not in snake_case", span.slice),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

pub(crate) fn check_screaming_snake_case(span: &Span, context: &mut ValidateContext<'_>) {
    if !is_screaming_snake_case(&span.slice) {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` identifier not in SCREAMING_SNAKE_CASE", span.slice),
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
