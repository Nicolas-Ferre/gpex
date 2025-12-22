use crate::compiler::indexes::Indexes;
use crate::compiletools::indexing::Node;
use crate::compiletools::parsing::Span;
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_found(
    node: &impl Node,
    ident: &Span,
    ctx: &mut ValidateCtx<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    let is_source_found = indexes.value_sources.contains_key(&node.id());
    if is_source_found {
        Ok(())
    } else {
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{}` value not found", ident.slice),
            loc: Some(ctx.loc(ident)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_unique_def(
    node: &impl Node,
    ident: &Span,
    ctx: &mut ValidateCtx<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if let Some(duplicated_item) = indexes.values.search(&ident.slice, node) {
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{}` item defined multiple times", ident.slice),
            loc: Some(ctx.loc(ident)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "item also defined here".into(),
                loc: Some(ctx.loc(&duplicated_item.ident)),
            }],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_usage(
    node: &impl Node,
    ident: &Span,
    ctx: &mut ValidateCtx<'_>,
    indexes: &Indexes<'_>,
) {
    let ref_span = indexes.item_first_ref.get(&node.id());
    if ref_span.is_none() && !ident.slice.starts_with('_') {
        ctx.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{}` value unused", ident.slice),
            loc: Some(ctx.loc(ident)),
            inner: vec![],
        });
    } else if let Some(ref_span) = ref_span
        && ident.slice.starts_with('_')
    {
        ctx.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{}` value used but name starting with `_`", ident.slice),
            loc: Some(ctx.loc(ident)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "value used here".into(),
                loc: Some(ctx.loc(ref_span)),
            }],
        });
    }
}
