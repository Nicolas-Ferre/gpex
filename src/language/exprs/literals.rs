use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::patterns::I32_LIT_PAT;
use crate::validators;

#[derive(Debug)]
pub(crate) struct I32Lit {
    span: Span,
    cleaned: String,
}

impl I32Lit {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        let span = Span::parse_pattern(ctx, I32_LIT_PAT)?;
        Ok(Self {
            cleaned: span.slice.replace('_', ""),
            span,
        })
    }

    pub(crate) fn validate(&self, ctx: &mut ValidateCtx<'_>) -> Result<(), ValidateError> {
        validators::literal::check_i32_bounds(&self.cleaned, &self.span, ctx)?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        *shader += "i32(";
        *shader += &self.cleaned;
        *shader += ")";
    }
}
