use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::symbols::{REPEAT_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct RepeatDefinition {
    pub(crate) call: Call,
}

impl RepeatDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        Span::parse_symbol(context, REPEAT_KEYWORD)?;
        context.force_parse_any_error();
        let call = Call::parse(context)?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self { call })
    }
}
