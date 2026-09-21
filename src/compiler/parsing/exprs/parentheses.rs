use crate::compiler::parsing::exprs;
use crate::compiler::transversal::ast::exprs::parentheses::ParenthesizedExpr;
use crate::compiler::transversal::ast::symbols::{
    PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL,
};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

pub(super) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<ParenthesizedExpr, ParseError<'context>> {
    let start_span = Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
    context.force_parse_any_error();
    let value = exprs::parse(context, |context| {
        Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ())
    })?;
    let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
    Ok(ParenthesizedExpr {
        span: start_span.until(end_span),
        value: Box::new(value),
    })
}
