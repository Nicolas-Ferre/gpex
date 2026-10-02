use crate::compiler::parsing::exprs::calls;
use crate::compiler::transversal::ast::items::actions::RepeatDefinition;
use crate::compiler::transversal::ast::symbols::{REPEAT_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<RepeatDefinition, ParseError<'context>> {
    Span::parse_symbol(context, REPEAT_KEYWORD)?;
    context.force_parse_any_error();
    let call = calls::parse(context)?;
    Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
    Ok(RepeatDefinition { call })
}
