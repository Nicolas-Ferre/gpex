use crate::compiler::indexes::{Indexes, Value};
use crate::compiletools::indexing::NodeRef;
use crate::compiletools::parsing::Span;
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_const(
    node: impl NodeRef,
    ident: &Span,
    const_span: &Span,
    ctx: &mut ValidateCtx<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if matches!(indexes.value_sources[&node.id()], Value::Const(_)) {
        Ok(())
    } else {
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: "expression not constant".into(),
            loc: Some(ctx.loc(ident)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "expression must be constant".into(),
                loc: Some(ctx.loc(const_span)),
            }],
        });
        Err(ValidateError)
    }
}

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

pub(crate) fn check_screaming_snake_case(ident: &Span, ctx: &mut ValidateCtx<'_>) {
    if !is_screaming_snake_case(&ident.slice) {
        ctx.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{}` identifier not in SCREAMING_SNAKE_CASE", ident.slice),
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

fn is_screaming_snake_case(ident: &str) -> bool {
    ident
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}
