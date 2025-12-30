use crate::compiler::EXTENSION;
use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};
use itertools::Itertools;
use std::path::PathBuf;

pub(crate) fn check_found(
    is_found: bool,
    segments: &[Span],
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    debug_assert!(!segments.is_empty());
    if is_found {
        Ok(())
    } else {
        let dot_path = segments.iter().map(|segment| &segment.slice).join(".");
        let file_path = segments
            .iter()
            .map(|segment| &segment.slice)
            .collect::<PathBuf>()
            .with_extension(EXTENSION);
        let full_path = context.root_path.join(file_path);
        let segments_span = segments[0].until(&segments[segments.len() - 1]);
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{dot_path}` module not found"),
            location: Some(context.location(&segments_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: format!("cannot read \"{}\"", full_path.display()),
                location: None,
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_not_top(
    is_top: bool,
    span: &Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if is_top {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "`import` statement not at the top of the module".into(),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "`import` statements should appear before anything else".into(),
                location: None,
            }],
        });
        Err(ValidateError)
    }
}
