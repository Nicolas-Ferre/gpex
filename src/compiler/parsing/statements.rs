use crate::compiler::parsing::exprs;
use crate::compiler::transversal::ast::statements::{
    AssignmentStatement, ReturnStatement, Statement,
};
use crate::compiler::transversal::ast::symbols::{EQUAL_SYMBOL, RETURN_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Statement, ParseError<'context>> {
    context.parse_any(&[
        &|context| parse_return_statement(context).map(Statement::Return),
        &|context| parse_assignment_statement(context).map(Statement::Assignment),
    ])
}

fn parse_return_statement<'context>(
    context: &mut ParseContext<'context>,
) -> Result<ReturnStatement, ParseError<'context>> {
    let return_keyword_span = Span::parse_symbol(context, RETURN_KEYWORD)?;
    context.force_parse_any_error();
    let value = exprs::parse(context, |context| {
        Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ())
    })?;
    let semicolon_span = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
    Ok(ReturnStatement {
        span: return_keyword_span.until(semicolon_span),
        value,
    })
}

fn parse_assignment_statement<'context>(
    context: &mut ParseContext<'context>,
) -> Result<AssignmentStatement, ParseError<'context>> {
    let assigned = exprs::parse(context, |context| {
        Span::parse_symbol(context, EQUAL_SYMBOL).map(|_| ())
    })?;
    Span::parse_symbol(context, EQUAL_SYMBOL)?;
    context.force_parse_any_error();
    let value = exprs::parse(context, |context| {
        Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ())
    })?;
    let semicolon_span = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
    Ok(AssignmentStatement {
        span: assigned.span().until(semicolon_span),
        assigned,
        value,
    })
}
