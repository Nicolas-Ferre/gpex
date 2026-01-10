use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::{F32_LITERAL_PATTERN, I32_LITERAL_PATTERN, U32_LITERAL_PATTERN};
use crate::language::symbols::{FALSE_KEYWORD, TRUE_KEYWORD};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
pub(crate) struct I32Literal {
    pub(crate) span: Span,
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

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index StructDefinition {
        indexes.search_prelude_type("i32")
    }

    pub(crate) fn constant<'index>(&self) -> Option<Constant<'index>> {
        self.value.map(Constant::I32)
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::literal::check_bounds(self.value.is_some(), self.span, "i32", context)?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        self.constant()
            .unwrap_or_else(|| unreachable!("literals should be validated before transpilation"))
            .transpile(shader);
    }
}

#[derive(Debug)]
pub(crate) struct U32Literal {
    pub(crate) span: Span,
    value: Option<u32>,
}

impl U32Literal {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, U32_LITERAL_PATTERN)?;
        Ok(Self {
            value: context
                .slice(span)
                .replace(['_', 'u'], "")
                .parse::<u32>()
                .ok(),
            span,
        })
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index StructDefinition {
        indexes.search_prelude_type("u32")
    }

    pub(crate) fn constant<'index>(&self) -> Option<Constant<'index>> {
        self.value.map(Constant::U32)
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::literal::check_bounds(self.value.is_some(), self.span, "u32", context)?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        self.constant()
            .unwrap_or_else(|| unreachable!("literals should be validated before transpilation"))
            .transpile(shader);
    }
}

#[derive(Debug)]
pub(crate) struct F32Literal {
    pub(crate) span: Span,
    value: Option<f32>,
}

impl F32Literal {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, F32_LITERAL_PATTERN)?;
        Ok(Self {
            value: context
                .slice(span)
                .replace('_', "")
                .parse::<f32>()
                .ok()
                .filter(|value| value.is_finite()),
            span,
        })
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index StructDefinition {
        indexes.search_prelude_type("f32")
    }

    pub(crate) fn constant<'index>(&self) -> Option<Constant<'index>> {
        self.value.map(Constant::F32)
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::literal::check_bounds(self.value.is_some(), self.span, "f32", context)?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        self.constant()
            .unwrap_or_else(|| unreachable!("literals should be validated before transpilation"))
            .transpile(shader);
    }
}

#[derive(Debug)]
pub(crate) struct BoolLiteral {
    pub(crate) span: Span,
    pub(crate) value: bool,
}

impl BoolLiteral {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = context.parse_any(&[
            |context| Span::parse_symbol(context, TRUE_KEYWORD),
            |context| Span::parse_symbol(context, FALSE_KEYWORD),
        ])?;
        Ok(Self {
            span,
            value: context.slice(span) == TRUE_KEYWORD.slice,
        })
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index StructDefinition {
        indexes.search_prelude_type("bool")
    }

    pub(crate) fn constant<'index>(&self) -> Constant<'index> {
        Constant::Bool(self.value)
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        self.constant().transpile(shader);
    }
}
