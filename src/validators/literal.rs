use crate::compiletools::parsing::Span;
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::{Log, LogLevel};

pub(crate) fn check_i32_bounds(
    value: &str,
    span: &Span,
    ctx: &mut ValidateCtx<'_>,
) -> Result<(), ValidateError> {
    if value.parse::<i32>().is_err() {
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: "`i32` literal out of bounds".into(),
            loc: Some(ctx.loc(span)),
            inner: vec![],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}
