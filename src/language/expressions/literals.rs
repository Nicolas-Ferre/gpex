use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::language::patterns::I32_LITERAL_PATTERN;
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
pub(crate) struct I32Literal {
    id: u64,
    span: Span,
    cleaned: String,
}

impl I32Literal {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, I32_LITERAL_PATTERN)?;
        Ok(Self {
            id: context.next_id(),
            cleaned: span.slice.replace('_', ""),
            span,
        })
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let value = validators::literal::check_i32_bounds(&self.cleaned, &self.span, context)?;
        indexes.constants.insert(self.id, Constant::I32(value));
        Ok(())
    }

    pub(crate) fn constant<'index>(&self, indexes: &'index Indexes<'_>) -> &'index Constant {
        &indexes.constants[&self.id]
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.constant(indexes).transpile(shader);
    }
}
