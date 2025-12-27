use crate::compiler::EXTENSION;
use crate::compiletools::parsing::Span;
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::{Log, LogInner, LogLevel};
use itertools::Itertools;
use std::path::PathBuf;

pub(crate) fn check_found(
    is_found: bool,
    segments: &[Span],
    ctx: &mut ValidateCtx<'_>,
) -> Result<(), ValidateError> {
    debug_assert!(!segments.is_empty());
    if is_found {
        Ok(())
    } else {
        let dot_path = segments.iter().map(|segment| &segment.slice).join(".");
        let file_path = segments
            .iter()
            .map(|segment| &segment.slice)
            .collect::<PathBuf>()
            .with_extension(EXTENSION);
        let full_path = ctx.root_path.join(file_path);
        let span = Span {
            file_index: segments[0].file_index,
            start: segments[0].start,
            end: segments[segments.len() - 1].end,
            slice: String::new(),
        };
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{dot_path}` module not found"),
            loc: Some(ctx.loc(&span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: format!("cannot read \"{}\"", full_path.display()),
                loc: None,
            }],
        });
        Err(ValidateError)
    }
}
