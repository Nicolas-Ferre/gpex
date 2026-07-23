use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogInner, LogLevel};
use std::path::Path;

pub(crate) fn not_found(
    dot_path: &str,
    dot_path_span: Span,
    fs_path: &Path,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{dot_path}` module not found"),
        location: Some(state.span_location(dot_path_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: format!("cannot read \"{}\"", fs_path.display()),
            location: None,
        }],
    }
}

pub(crate) fn not_at_top(import_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "`import` statement not at the top of the module".into(),
        location: Some(state.span_location(import_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "`import` statements should appear before anything else".into(),
            location: None,
        }],
    }
}

pub(crate) fn self_import(import_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: "module importing itself".into(),
        location: Some(state.span_location(import_span)),
        inner: vec![],
    }
}

pub(crate) fn unused(dot_path: &str, import_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: format!("`{dot_path}` import unused"),
        location: Some(state.span_location(import_span)),
        inner: vec![],
    }
}
