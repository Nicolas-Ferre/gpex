use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::symbols::{EQUAL_SYMBOL, RETURN_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) enum Statement {
    Return(ReturnStatement),
    Assignment(AssignmentStatement),
}

impl Statement {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| ReturnStatement::parse(context).map(Self::Return),
            |context| AssignmentStatement::parse(context).map(Self::Assignment),
        ])
    }
}

#[derive(Debug)]
pub(crate) struct ReturnStatement {
    pub(crate) span: Span,
    pub(crate) value: Expr,
}

impl ReturnStatement {
    fn parse<'context>(context: &mut ParseContext<'context>) -> Result<Self, ParseError<'context>> {
        let return_keyword_span = Span::parse_symbol(context, RETURN_KEYWORD)?;
        context.force_parse_any_error();
        let value = Expr::parse(context, |context| {
            Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ())
        })?;
        let semicolon_keyword_span = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self {
            span: return_keyword_span.until(semicolon_keyword_span),
            value,
        })
    }
}

#[derive(Debug)]
pub(crate) struct AssignmentStatement {
    pub(crate) assigned: Expr,
    pub(crate) value: Expr,
}

impl AssignmentStatement {
    fn parse<'context>(context: &mut ParseContext<'context>) -> Result<Self, ParseError<'context>> {
        let assigned = Expr::parse(context, |context| {
            Span::parse_symbol(context, EQUAL_SYMBOL).map(|_| ())
        })?;
        Span::parse_symbol(context, EQUAL_SYMBOL)?;
        context.force_parse_any_error();
        let value = Expr::parse(context, |context| {
            Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ())
        })?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self { assigned, value })
    }
}
