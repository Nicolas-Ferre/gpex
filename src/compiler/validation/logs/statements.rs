use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogInner, LogLevel};

pub(crate) fn return_before_end(
    return_span: Span,
    next_statement_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "`return` statement not at the end of the block".into(),
        location: Some(state.span_location(return_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "this statement is after".into(),
            location: Some(state.span_location(next_statement_span)),
        }],
    }
}

pub(crate) fn missing_return(
    previous_statement_span: Span,
    return_type_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "missing `return` statement".into(),
        location: Some(state.span_location(previous_statement_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "function has a return type".into(),
            location: Some(state.span_location(return_type_span)),
        }],
    }
}

pub(crate) fn disallowed_return(
    return_span: Span,
    fn_signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "`return` statement in function with no return type".into(),
        location: Some(state.span_location(return_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "function has no return type".into(),
            location: Some(state.span_location(fn_signature_span)),
        }],
    }
}

pub(crate) fn empty_block(block_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: "empty statement block".into(),
        location: Some(state.span_location(block_span)),
        inner: vec![],
    }
}
