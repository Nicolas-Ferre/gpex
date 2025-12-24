use crate::compiletools::parsing::Span;
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::{Log, LogLevel};

pub(crate) fn check_i32_bounds(
    value: &str,
    span: &Span,
    ctx: &mut ValidateCtx<'_>,
) -> Result<i32, ValidateError> {
    if let Ok(value) = value.parse::<i32>() {
        Ok(value)
    } else {
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: "`i32` literal out of bounds".into(),
            loc: Some(ctx.loc(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}
