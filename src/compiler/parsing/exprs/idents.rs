use crate::compiler::transversal::ast::exprs::idents::Ident;
use crate::compiler::transversal::ast::patterns::IDENT_PATTERN;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Ident, ParseError<'context>> {
    let span = Span::parse_pattern(context, IDENT_PATTERN)?;
    Ok(Ident {
        id: context.next_id(),
        scope: context.scope().to_vec(),
        slice: context.slice(span).into(),
        span,
    })
}
