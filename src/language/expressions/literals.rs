use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::{I32_LITERAL_PATTERN, U32_LITERAL_PATTERN};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
pub(crate) struct I32Literal {
    span: Span,
    value: Option<i32>,
}

impl I32Literal {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, I32_LITERAL_PATTERN)?;
        Ok(Self {
            value: context.slice(span).replace('_', "").parse::<i32>().ok(),
            span,
        })
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::literal::check_i32_bounds(self.value, self.span, context)?;
        Ok(())
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index StructDefinition {
        indexes.search_prelude_type("i32")
    }

    #[expect(clippy::expect_used)] // validated during previous pass
    pub(crate) fn constant<'index>(&self) -> Constant<'index> {
        Constant::I32(self.value.expect("internal error: invalid `i32` literal"))
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        self.constant().transpile(shader);
    }
}

#[derive(Debug)]
pub(crate) struct U32Literal {
    span: Span,
    value: Option<u32>,
}

impl U32Literal {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, U32_LITERAL_PATTERN)?;
        let slice = context.slice(span);
        Ok(Self {
            value: slice[..slice.len() - 1]
                .replace('_', "")
                .parse::<u32>()
                .ok(),
            span,
        })
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::literal::check_u32_bounds(self.value, self.span, context)?;
        Ok(())
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index StructDefinition {
        indexes.search_prelude_type("u32")
    }

    #[expect(clippy::expect_used)] // validated during previous pass
    pub(crate) fn constant<'index>(&self) -> Constant<'index> {
        Constant::U32(self.value.expect("internal error: invalid `u32` literal"))
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        self.constant().transpile(shader);
    }
}
