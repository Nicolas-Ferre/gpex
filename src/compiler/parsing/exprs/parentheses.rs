use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::symbols::{PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct ParenthesizedExpr {
    pub(crate) span: Span,
    pub(crate) value: Box<Expr>,
}

impl ParenthesizedExpr {
    pub(super) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let start_span = Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
        context.force_parse_any_error();
        let value = Expr::parse(context, |context| {
            Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ())
        })?;
        let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
        Ok(Self {
            span: start_span.until(end_span),
            value: Box::new(value),
        })
    }
}
