pub(crate) mod calls;
pub(crate) mod idents;
pub(crate) mod literals;
pub(crate) mod parentheses;

use crate::compiler::transversal::ast::exprs::Expr;
use crate::compiler::transversal::ast::exprs::calls::Call;
use crate::compiler::transversal::ast::symbols::{
    AND_SYMBOL, ANGLE_BRACKET_CLOSE_SYMBOL, ANGLE_BRACKET_OPEN_SYMBOL, COMPARE_EQUAL_SYMBOL,
    COMPARE_GREATER_EQUAL_SYMBOL, COMPARE_LESS_EQUAL_SYMBOL, COMPARE_NOT_EQUAL_SYMBOL, DOT_SYMBOL,
    HYPHEN_SYMBOL, OR_SYMBOL, PERCENT_SYMBOL, PLUS_SYMBOL, QUESTION_MARK_SYMBOL, SLASH_SYMBOL,
    STAR_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, Parser, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

const OPERATOR_PRIORITIES: &[&[&str]] = &[
    &[STAR_SYMBOL.slice, SLASH_SYMBOL.slice, PERCENT_SYMBOL.slice],
    &[PLUS_SYMBOL.slice, HYPHEN_SYMBOL.slice],
    &[
        COMPARE_EQUAL_SYMBOL.slice,
        COMPARE_NOT_EQUAL_SYMBOL.slice,
        COMPARE_LESS_EQUAL_SYMBOL.slice,
        ANGLE_BRACKET_OPEN_SYMBOL.slice,
        COMPARE_GREATER_EQUAL_SYMBOL.slice,
        ANGLE_BRACKET_CLOSE_SYMBOL.slice,
    ],
    &[AND_SYMBOL.slice],
    &[OR_SYMBOL.slice],
];

struct BinaryRightPart {
    operator: Span,
    operand: Expr,
}

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
    stop_excluded_parser: Parser<'context, ()>,
) -> Result<Expr, ParseError<'context>> {
    let left_operand = parse_operand(context, stop_excluded_parser)?;
    let binary_right_parts = context.parse_many(
        |context| parse_binary_right_part(context, stop_excluded_parser),
        SeparatorParser::None,
        stop_excluded_parser,
    )?;
    Ok(create_binary_chain(
        context,
        left_operand,
        binary_right_parts,
    ))
}

fn parse_binary_right_part<'context>(
    context: &mut ParseContext<'context>,
    stop_excluded_parser: Parser<'context, ()>,
) -> Result<BinaryRightPart, ParseError<'context>> {
    let operator = parse_binary_operator(context)?;
    context.force_parse_any_error();
    let operand = parse_operand(context, stop_excluded_parser)?;
    Ok(BinaryRightPart { operator, operand })
}

fn parse_operand<'context>(
    context: &mut ParseContext<'context>,
    stop_excluded_parser: Parser<'context, ()>,
) -> Result<Expr, ParseError<'context>> {
    let mut expr = parse_operand_prefix(context, stop_excluded_parser)?;
    let parsed_calls =
        context.parse_many(parse_operand_suffix, SeparatorParser::None, |context| {
            parse_operand_stop(context, stop_excluded_parser)
        })?;
    for call in parsed_calls {
        expr = Expr::Call(calls::from_uniform_syntax(expr, call));
    }
    Ok(expr)
}

fn parse_operand_prefix<'context>(
    context: &mut ParseContext<'context>,
    stop_excluded_parser: Parser<'context, ()>,
) -> Result<Expr, ParseError<'context>> {
    context.parse_any(&[
        &|context| literals::parse_f32_literal(context).map(Expr::F32Literal),
        &|context| literals::parse_u32_literal(context).map(Expr::U32Literal),
        &|context| literals::parse_i32_literal(context).map(Expr::I32Literal),
        &|context| literals::parse_bool_literal(context).map(Expr::BoolLiteral),
        &|context| Span::parse_symbol(context, QUESTION_MARK_SYMBOL).map(Expr::Wildcard),
        &|context| calls::parse(context).map(Expr::Call),
        &|context| calls::parse_unary(context, stop_excluded_parser).map(Expr::Call),
        &|context| parentheses::parse(context).map(Expr::Parenthesized),
        &|context| idents::parse(context).map(Expr::Ident),
    ])
}

fn parse_operand_suffix<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Call, ParseError<'context>> {
    Span::parse_symbol(context, DOT_SYMBOL)?;
    calls::parse(context)
}

fn parse_operand_stop<'context>(
    context: &mut ParseContext<'context>,
    stop_excluded_parser: Parser<'context, ()>,
) -> Result<(), ParseError<'context>> {
    context.parse_any(&[
        &|context| parse_binary_operator(context).map(|_| ()),
        &stop_excluded_parser,
    ])
}

fn parse_binary_operator<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Span, ParseError<'context>> {
    context.parse_any(&[
        &|context| Span::parse_symbol(context, PLUS_SYMBOL),
        &|context| Span::parse_symbol(context, HYPHEN_SYMBOL),
        &|context| Span::parse_symbol(context, STAR_SYMBOL),
        &|context| Span::parse_symbol(context, SLASH_SYMBOL),
        &|context| Span::parse_symbol(context, PERCENT_SYMBOL),
        &|context| Span::parse_symbol(context, COMPARE_EQUAL_SYMBOL),
        &|context| Span::parse_symbol(context, COMPARE_NOT_EQUAL_SYMBOL),
        &|context| Span::parse_symbol(context, COMPARE_LESS_EQUAL_SYMBOL),
        &|context| Span::parse_symbol(context, ANGLE_BRACKET_OPEN_SYMBOL),
        &|context| Span::parse_symbol(context, COMPARE_GREATER_EQUAL_SYMBOL),
        &|context| Span::parse_symbol(context, ANGLE_BRACKET_CLOSE_SYMBOL),
        &|context| Span::parse_symbol(context, AND_SYMBOL),
        &|context| Span::parse_symbol(context, OR_SYMBOL),
    ])
}

fn create_binary_chain(
    context: &mut ParseContext<'_>,
    left_operand: Expr,
    mut binary_right_parts: Vec<BinaryRightPart>,
) -> Expr {
    if binary_right_parts.is_empty() {
        return left_operand;
    }
    let mut remaining_right_parts =
        binary_right_parts.split_off(lowest_priority_operator_index(context, &binary_right_parts));
    let first_remaining_right_part = remaining_right_parts.remove(0);
    let left_operand = create_binary_chain(context, left_operand, binary_right_parts);
    let right_operand = create_binary_chain(
        context,
        first_remaining_right_part.operand,
        remaining_right_parts,
    );
    Expr::Call(calls::from_binary(
        context,
        left_operand,
        first_remaining_right_part.operator,
        right_operand,
    ))
}

fn lowest_priority_operator_index(
    context: &ParseContext<'_>,
    binary_right_parts: &[BinaryRightPart],
) -> usize {
    OPERATOR_PRIORITIES
        .iter()
        .rev()
        .find_map(|operators| {
            binary_right_parts
                .iter()
                .rposition(|part| operators.contains(&context.slice(part.operator)))
        })
        .unwrap_or_else(|| unreachable!("no supported binary operator found"))
}
