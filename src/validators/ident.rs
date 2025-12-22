use crate::compiletools::parsing::Span;
use crate::compiletools::validation::ValidateCtx;
use crate::{Log, LogLevel};

pub(crate) fn check_letter_count(ident: &Span, ctx: &mut ValidateCtx<'_>) {
    if ident.slice.len() == 1 && ident.slice != "_" {
        ctx.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{}` identifier is single letter", ident.slice),
            loc: Some(ctx.loc(ident)),
            inner: vec![],
        });
    }
}

pub(crate) fn check_snake_case(ident: &Span, ctx: &mut ValidateCtx<'_>) {
    if !is_snake_case(&ident.slice) {
        ctx.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{}` identifier not in snake_case", ident.slice),
            loc: Some(ctx.loc(ident)),
            inner: vec![],
        });
    }
}

fn is_snake_case(ident: &str) -> bool {
    ident
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
