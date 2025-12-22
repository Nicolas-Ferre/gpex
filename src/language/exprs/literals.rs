use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::patterns::I32_LIT_PAT;
use crate::validators;

#[derive(Debug)]
pub(crate) struct I32Lit {
    span: Span,
}

impl I32Lit {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        Ok(Self {
            span: Span::parse_pattern(ctx, I32_LIT_PAT)?,
        })
    }

    pub(crate) fn validate(&self, ctx: &mut ValidateCtx<'_>) -> Result<(), ValidateError> {
        validators::literal::check_i32_bounds(&self.cleaned_slice(), &self.span, ctx)?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        *shader += "i32(";
        *shader += &self.cleaned_slice();
        *shader += ")";
    }

    fn cleaned_slice(&self) -> String {
        self.span.slice.replace('_', "")
    }
}
