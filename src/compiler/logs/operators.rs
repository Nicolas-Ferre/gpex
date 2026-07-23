use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogLevel};

pub(crate) fn unary_operator_param_count(
    fn_key: &str,
    fn_signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{fn_key}` unary operator function must have exactly one parameter"),
        location: Some(state.span_location(fn_signature_span)),
        inner: vec![],
    }
}

pub(crate) fn unary_operator_without_return_type(
    fn_key: &str,
    fn_signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{fn_key}` unary operator function without return type"),
        location: Some(state.span_location(fn_signature_span)),
        inner: vec![],
    }
}

pub(crate) fn binary_operator_param_count(
    fn_key: &str,
    fn_signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{fn_key}` binary operator function must have exactly two parameters"),
        location: Some(state.span_location(fn_signature_span)),
        inner: vec![],
    }
}

pub(crate) fn binary_operator_without_return_type(
    fn_key: &str,
    fn_signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{fn_key}` binary operator function without return type"),
        location: Some(state.span_location(fn_signature_span)),
        inner: vec![],
    }
}
