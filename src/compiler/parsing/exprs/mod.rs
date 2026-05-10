pub(crate) mod calls;
pub(crate) mod idents;
pub(crate) mod literals;

use crate::compiler::parsing::symbols::{
    AND_SYMBOL, ANGLE_BRACKET_CLOSE_SYMBOL, ANGLE_BRACKET_OPEN_SYMBOL, COMPARE_EQUAL_SYMBOL,
    COMPARE_GREATER_EQUAL_SYMBOL, COMPARE_LESS_EQUAL_SYMBOL, COMPARE_NOT_EQUAL_SYMBOL,
    HYPHEN_SYMBOL, OR_SYMBOL, PERCENT_SYMBOL, PLUS_SYMBOL, SLASH_SYMBOL, STAR_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, Parser, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use calls::Call;
use idents::Ident;
use literals::{BoolLiteral, F32Literal, I32Literal, U32Literal};

const BINARY_ADD_FN_NAME: &str = "__add__";
const BINARY_SUB_FN_NAME: &str = "__sub__";
const BINARY_MUL_FN_NAME: &str = "__mul__";
const BINARY_DIV_FN_NAME: &str = "__div__";
const BINARY_MOD_FN_NAME: &str = "__mod__";
const BINARY_EQ_FN_NAME: &str = "__eq__";
const BINARY_NE_FN_NAME: &str = "__ne__";
const BINARY_LT_FN_NAME: &str = "__lt__";
const BINARY_LE_FN_NAME: &str = "__le__";
const BINARY_GT_FN_NAME: &str = "__gt__";
const BINARY_GE_FN_NAME: &str = "__ge__";
const BINARY_AND_FN_NAME: &str = "__and__";
const BINARY_OR_FN_NAME: &str = "__or__";
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

#[derive(Debug)]
pub(crate) enum Expr {
    F32Literal(F32Literal),
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    BoolLiteral(BoolLiteral),
    Call(Call),
    Ident(Ident),
}

impl Expr {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<Self, ParseError<'context>> {
        let left_operand = Self::parse_operand(context)?;
        let binary_right_parts = context.parse_many(
            Self::parse_binary_right_part,
            SeparatorParser::None,
            stop_excluded_parser,
        )?;
        Ok(Self::create_binary_call(
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
            Self::Call(node) => node.span,
            Self::Ident(node) => node.span,
        }
    }

    fn parse_operand<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| F32Literal::parse(context).map(Self::F32Literal),
            |context| U32Literal::parse(context).map(Self::U32Literal),
            |context| I32Literal::parse(context).map(Self::I32Literal),
            |context| BoolLiteral::parse(context).map(Self::BoolLiteral),
            |context| Call::parse(context).map(Self::Call),
            |context| Call::parse_unary(context).map(Self::Call),
            |context| Ident::parse(context).map(Self::Ident),
        ])
    }

    fn parse_binary_right_part<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<BinaryRightPart, ParseError<'context>> {
        let operator = context.parse_any(&[
            |context| Span::parse_symbol(context, PLUS_SYMBOL),
            |context| Span::parse_symbol(context, HYPHEN_SYMBOL),
            |context| Span::parse_symbol(context, STAR_SYMBOL),
            |context| Span::parse_symbol(context, SLASH_SYMBOL),
            |context| Span::parse_symbol(context, PERCENT_SYMBOL),
            |context| Span::parse_symbol(context, COMPARE_EQUAL_SYMBOL),
            |context| Span::parse_symbol(context, COMPARE_NOT_EQUAL_SYMBOL),
            |context| Span::parse_symbol(context, COMPARE_LESS_EQUAL_SYMBOL),
            |context| Span::parse_symbol(context, ANGLE_BRACKET_OPEN_SYMBOL),
            |context| Span::parse_symbol(context, COMPARE_GREATER_EQUAL_SYMBOL),
            |context| Span::parse_symbol(context, ANGLE_BRACKET_CLOSE_SYMBOL),
            |context| Span::parse_symbol(context, AND_SYMBOL),
            |context| Span::parse_symbol(context, OR_SYMBOL),
        ])?;
        context.force_parse_any_error();
        let operand = Self::parse_operand(context)?;
        Ok(BinaryRightPart { operator, operand })
    }

    fn create_binary_call(
        context: &mut ParseContext<'_>,
        mut left_operand: Self,
        binary_right_parts: Vec<BinaryRightPart>,
    ) -> Self {
        for binary_right_part in binary_right_parts {
            left_operand = Self::Call(Call::from_binary(context, left_operand, binary_right_part));
        }
        left_operand
    }
}

struct BinaryRightPart {
    operator: Span,
    operand: Expr,
}
