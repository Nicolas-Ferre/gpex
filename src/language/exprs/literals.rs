use crate::compiler::constants::ConstValue;
use crate::compiler::indexes::Indexes;
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::patterns::I32_LIT_PAT;
use crate::validators;

#[derive(Debug)]
pub(crate) struct I32Lit {
    id: u64,
    span: Span,
    cleaned: String,
}

impl I32Lit {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        let span = Span::parse_pattern(ctx, I32_LIT_PAT)?;
        Ok(Self {
            id: ctx.next_id(),
            cleaned: span.slice.replace('_', ""),
            span,
        })
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let value = validators::literal::check_i32_bounds(&self.cleaned, &self.span, ctx)?;
        indexes.const_values.insert(self.id, ConstValue::I32(value));
        Ok(())
    }

    pub(crate) fn const_value<'a>(&self, indexes: &'a Indexes<'_>) -> &'a ConstValue {
        &indexes.const_values[&self.id]
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.const_value(indexes).transpile(shader);
    }
}
