use crate::compiler::parsing::patterns::{
    F32_LITERAL_PATTERN, I32_LITERAL_PATTERN, U32_LITERAL_PATTERN,
};
use crate::compiler::parsing::symbols::{FALSE_KEYWORD, TRUE_KEYWORD};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

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
}

#[derive(Debug)]
pub(crate) struct F32Literal {
    pub(crate) span: Span,
    pub(crate) value: Option<f32>,
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
}

#[derive(Debug)]
pub(crate) struct I32Literal {
    pub(crate) span: Span,
    pub(crate) value: Option<i32>,
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
}

#[derive(Debug)]
pub(crate) struct U32Literal {
    pub(crate) span: Span,
    pub(crate) value: Option<u32>,
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
}
