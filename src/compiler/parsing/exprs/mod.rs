pub(crate) mod calls;
pub(crate) mod idents;
pub(crate) mod literals;

use crate::compiler::parsing::symbols::{
    AND_SYMBOL, ANGLE_BRACKET_CLOSE_SYMBOL, ANGLE_BRACKET_OPEN_SYMBOL, COMPARE_EQUAL_SYMBOL,
    COMPARE_GREATER_EQUAL_SYMBOL, COMPARE_LESS_EQUAL_SYMBOL, COMPARE_NOT_EQUAL_SYMBOL, DOT_SYMBOL,
    HYPHEN_SYMBOL, OR_SYMBOL, PERCENT_SYMBOL, PLUS_SYMBOL, QUESTION_MARK_SYMBOL, SLASH_SYMBOL,
    STAR_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, Parser, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use calls::Call;
use idents::Ident;
use literals::{BoolLiteral, F32Literal, I32Literal, U32Literal};

pub(crate) const BINARY_ADD_FN_NAME: &str = "__add__";
pub(crate) const BINARY_SUB_FN_NAME: &str = "__sub__";
pub(crate) const BINARY_MUL_FN_NAME: &str = "__mul__";
pub(crate) const BINARY_DIV_FN_NAME: &str = "__div__";
pub(crate) const BINARY_MOD_FN_NAME: &str = "__mod__";
pub(crate) const BINARY_EQ_FN_NAME: &str = "__eq__";
pub(crate) const BINARY_NE_FN_NAME: &str = "__ne__";
pub(crate) const BINARY_LT_FN_NAME: &str = "__lt__";
pub(crate) const BINARY_LE_FN_NAME: &str = "__le__";
pub(crate) const BINARY_GT_FN_NAME: &str = "__gt__";
pub(crate) const BINARY_GE_FN_NAME: &str = "__ge__";
pub(crate) const BINARY_AND_FN_NAME: &str = "__and__";
pub(crate) const BINARY_OR_FN_NAME: &str = "__or__";
pub(crate) const BINARY_FN_NAMES: &[&str] = &[
    BINARY_ADD_FN_NAME,
    BINARY_SUB_FN_NAME,
    BINARY_MUL_FN_NAME,
    BINARY_DIV_FN_NAME,
    BINARY_MOD_FN_NAME,
    BINARY_EQ_FN_NAME,
    BINARY_NE_FN_NAME,
    BINARY_LT_FN_NAME,
    BINARY_LE_FN_NAME,
    BINARY_GT_FN_NAME,
    BINARY_GE_FN_NAME,
    BINARY_AND_FN_NAME,
    BINARY_OR_FN_NAME,
];
pub(crate) const OPERATOR_FN_NAME_PREFIX: &str = "__";
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

#[derive(Debug)]
pub(crate) enum Expr {
    F32Literal(F32Literal),
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    BoolLiteral(BoolLiteral),
    Wildcard(Span),
    Call(Call),
    Ident(Ident),
}

impl Expr {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<Self, ParseError<'context>> {
        let left_operand = Self::parse_operand(context, stop_excluded_parser)?;
        let binary_right_parts = context.parse_many(
            |context| Self::parse_binary_right_part(context, stop_excluded_parser),
            SeparatorParser::None,
            stop_excluded_parser,
        )?;
        Ok(Self::create_binary_chain(
            context,
            left_operand,
            binary_right_parts,
        ))
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::F32Literal(node) => node.span,
            Self::U32Literal(node) => node.span,
            Self::I32Literal(node) => node.span,
            Self::BoolLiteral(node) => node.span,
            Self::Wildcard(span) => *span,
            Self::Call(node) => node.span,
            Self::Ident(node) => node.span,
        }
    }

    fn parse_binary_right_part<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<BinaryRightPart, ParseError<'context>> {
        let operator = Self::parse_binary_operator(context)?;
        context.force_parse_any_error();
        let operand = Self::parse_operand(context, stop_excluded_parser)?;
        Ok(BinaryRightPart { operator, operand })
    }

    fn parse_operand<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<Self, ParseError<'context>> {
        let mut expr = Self::parse_operand_prefix(context, stop_excluded_parser)?;
        let calls = context.parse_many(
            Self::parse_operand_suffix,
            SeparatorParser::None,
            |context| Self::parse_operand_stop(context, stop_excluded_parser),
        )?;
        for call in calls {
            expr = Self::Call(Call::from_uniform_syntax(expr, call));
        }
        Ok(expr)
    }

    fn parse_operand_prefix<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            &|context| F32Literal::parse(context).map(Self::F32Literal),
            &|context| U32Literal::parse(context).map(Self::U32Literal),
            &|context| I32Literal::parse(context).map(Self::I32Literal),
            &|context| BoolLiteral::parse(context).map(Self::BoolLiteral),
            &|context| Span::parse_symbol(context, QUESTION_MARK_SYMBOL).map(Self::Wildcard),
            &|context| Call::parse(context).map(Self::Call),
            &|context| Call::parse_unary(context, stop_excluded_parser).map(Self::Call),
            &|context| Ident::parse(context).map(Self::Ident),
        ])
    }

    fn parse_operand_suffix<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Call, ParseError<'context>> {
        Span::parse_symbol(context, DOT_SYMBOL)?;
        Call::parse(context)
    }

    fn parse_operand_stop<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<(), ParseError<'context>> {
        context.parse_any(&[
            &|context| Self::parse_binary_operator(context).map(|_| ()),
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
        left_operand: Self,
        mut binary_right_parts: Vec<BinaryRightPart>,
    ) -> Self {
        if binary_right_parts.is_empty() {
            return left_operand;
        }
        let mut remaining_right_parts = binary_right_parts.split_off(
            Self::lowest_priority_operator_index(context, &binary_right_parts),
        );
        let first_remaining_right_part = remaining_right_parts.remove(0);
        let left_operand = Self::create_binary_chain(context, left_operand, binary_right_parts);
        let right_operand = Self::create_binary_chain(
            context,
            first_remaining_right_part.operand,
            remaining_right_parts,
        );
        Self::Call(Call::from_binary(
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
}

struct BinaryRightPart {
    operator: Span,
    operand: Expr,
}
