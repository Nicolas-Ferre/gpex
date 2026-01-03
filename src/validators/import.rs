use crate::compiler::indexes::Indexes;
use crate::language::import::ImportSegment;
use crate::utils::parsing::{Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};
use itertools::Itertools;

pub(crate) fn check_found(
    is_found: bool,
    segments: &[ImportSegment],
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    debug_assert!(!segments.is_empty());
    if is_found {
        Ok(())
    } else {
        let dot_path = dot_path_from_segments(segments, context);
        let fs_path = ImportSegment::fs_path(segments, context, context.root_path);
        let first_segment = segments[0];
        let last_segment = segments[segments.len() - 1];
        let segments_span = first_segment.span().until(last_segment.span());
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{dot_path}` module not found"),
            location: Some(context.location(segments_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: format!("cannot read \"{}\"", fs_path.display()),
                location: None,
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_top(
    is_top: bool,
    span: Span,
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

pub(crate) fn check_self_import(
    imported_file_index: Option<usize>,
    span: Span,
    context: &mut ValidateContext<'_>,
) {
    if let Some(imported_file_index) = imported_file_index
        && imported_file_index == span.file_index
    {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: "module importing itself".into(),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

pub(crate) fn check_usage(
    import_id: u64,
    imported_file_index: Option<usize>,
    span: Span,
    is_public: bool,
    segments: &[ImportSegment],
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    let is_self_import = imported_file_index == Some(span.file_index);
    if !is_self_import && !is_public && !indexes.imports.is_used(span.file_index, import_id) {
        let dot_path = dot_path_from_segments(segments, context);
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{dot_path}` import unused"),
            location: Some(context.location(span)),
            inner: vec![],
        });
    }
}

fn dot_path_from_segments(segments: &[ImportSegment], context: &ValidateContext<'_>) -> String {
    segments
        .iter()
        .map(|&segment| context.slice(segment.span()))
        .join(".")
}
