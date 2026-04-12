use crate::compiler::indexes::Indexes;
use crate::language::exprs::call::Call;
use crate::language::symbols::{REPEAT_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

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

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.call.index(indexes);
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.call.validate(None, context, indexes)?;
        validators::expr::check_has_return_type(&self.call, self.call.span, context, indexes)?;
        Ok(())
    }

    pub(crate) fn transpile_call(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.call.transpile(shader, indexes);
        *shader += "; ";
    }
}
