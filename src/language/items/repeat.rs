use crate::compiler::indexes::Indexes;
use crate::language::expressions::function_call::FunctionCall;
use crate::language::symbols::{REPEAT_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
pub(crate) struct RepeatDefinition {
    pub(crate) function_call: FunctionCall,
}

impl RepeatDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        Span::parse_symbol(context, REPEAT_KEYWORD)?;
        let function_call = FunctionCall::parse(context)?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self { function_call })
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.function_call.index(indexes);
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.function_call.validate(None, context, indexes)?;
        validators::expression::check_has_return_type(
            &self.function_call,
            self.function_call.span,
            context,
            indexes,
        )?;
        Ok(())
    }

    pub(crate) fn transpile_call(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.function_call.transpile(shader, indexes);
        *shader += "; ";
    }
}
