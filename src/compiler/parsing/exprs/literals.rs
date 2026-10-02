use crate::compiler::transversal::ast::exprs::literals::{
    BoolLiteral, F32Literal, I32Literal, U32Literal,
};
use crate::compiler::transversal::ast::patterns::{
    F32_LITERAL_PATTERN, I32_LITERAL_PATTERN, U32_LITERAL_PATTERN,
};
use crate::compiler::transversal::ast::symbols::{FALSE_KEYWORD, TRUE_KEYWORD};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn parse_bool_literal<'context>(
    context: &mut ParseContext<'context>,
) -> Result<BoolLiteral, ParseError<'context>> {
    let span = context.parse_any(&[
        &|context| Span::parse_symbol(context, TRUE_KEYWORD),
        &|context| Span::parse_symbol(context, FALSE_KEYWORD),
    ])?;
    Ok(BoolLiteral {
        span,
        value: context.slice(span) == TRUE_KEYWORD.slice,
    })
}

pub(crate) fn parse_f32_literal<'context>(
    context: &mut ParseContext<'context>,
) -> Result<F32Literal, ParseError<'context>> {
    let span = Span::parse_pattern(context, F32_LITERAL_PATTERN)?;
    Ok(F32Literal {
        value: context
            .slice(span)
            .replace('_', "")
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite()),
        span,
    })
}

pub(crate) fn parse_i32_literal<'context>(
    context: &mut ParseContext<'context>,
) -> Result<I32Literal, ParseError<'context>> {
    let span = Span::parse_pattern(context, I32_LITERAL_PATTERN)?;
    Ok(I32Literal {
        value: context.slice(span).replace('_', "").parse::<i32>().ok(),
        span,
    })
}

pub(crate) fn parse_u32_literal<'context>(
    context: &mut ParseContext<'context>,
) -> Result<U32Literal, ParseError<'context>> {
    let span = Span::parse_pattern(context, U32_LITERAL_PATTERN)?;
    Ok(U32Literal {
        value: context
            .slice(span)
            .replace(['_', 'u'], "")
            .parse::<u32>()
            .ok(),
        span,
    })
}
