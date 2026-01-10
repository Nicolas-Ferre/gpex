use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_return(
    span: Span,
    position: usize,
    statement_count: usize,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if position == statement_count - 1 {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "`return` statement not at the end of the block".into(),
            location: Some(context.location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_missing_return(
    is_missing: bool,
    block_end_span: Span,
    return_type_span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if is_missing {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "missing `return` statement".into(),
            location: Some(context.location(block_end_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "function has a return type".into(),
                location: Some(context.location(return_type_span)),
            }],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}
