use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogInner, LogLevel};

pub(crate) fn arg_name_mismatch(
    arg_name: &str,
    arg_name_span: Option<Span>,
    param_name: &str,
    param_name_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{arg_name}` argument name not matching parameter"),
        location: arg_name_span.map(|span| state.span_location(span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: format!("expected `{param_name}` parameter name"),
            location: Some(state.span_location(param_name_span)),
        }],
    }
}

pub(crate) fn called_fn_without_return_type(
    fn_key: &str,
    call_span: Span,
    signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("called function `{fn_key}` with no return type"),
        location: Some(state.span_location(call_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "function has no return type".into(),
            location: Some(state.span_location(signature_span)),
        }],
    }
}

pub(crate) fn repeated_fn_with_return_type(
    fn_key: &str,
    call_span: Span,
    signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("repeated function `{fn_key}` with a return type"),
        location: Some(state.span_location(call_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "function has a return type".into(),
            location: Some(state.span_location(signature_span)),
        }],
    }
}
