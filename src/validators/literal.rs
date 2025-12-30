use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogLevel};

pub(crate) fn check_i32_bounds(
    value: &str,
    span: &Span,
    context: &mut ValidateContext<'_>,
) -> Result<i32, ValidateError> {
    if let Ok(value) = value.parse::<i32>() {
        Ok(value)
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
