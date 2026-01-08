use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogLevel};

pub(crate) fn check_i32_bounds(
    value: Option<i32>,
    span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if value.is_some() {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "`i32` literal out of bounds".into(),
            location: Some(context.location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}
