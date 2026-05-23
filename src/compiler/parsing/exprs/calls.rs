use crate::compiler::parsing;
use crate::compiler::parsing::exprs::{
    BINARY_ADD_FN_NAME, BINARY_AND_FN_NAME, BINARY_DIV_FN_NAME, BINARY_EQ_FN_NAME,
    BINARY_GE_FN_NAME, BINARY_GT_FN_NAME, BINARY_LE_FN_NAME, BINARY_LT_FN_NAME, BINARY_MOD_FN_NAME,
    BINARY_MUL_FN_NAME, BINARY_NE_FN_NAME, BINARY_OR_FN_NAME, BINARY_SUB_FN_NAME, Expr,
};
use crate::compiler::parsing::patterns::IDENT_PATTERN;
use crate::compiler::parsing::symbols::{
    AND_SYMBOL, ANGLE_BRACKET_CLOSE_SYMBOL, ANGLE_BRACKET_OPEN_SYMBOL, COMMA_SYMBOL,
    COMPARE_EQUAL_SYMBOL, COMPARE_GREATER_EQUAL_SYMBOL, COMPARE_LESS_EQUAL_SYMBOL,
    COMPARE_NOT_EQUAL_SYMBOL, EXCLAMATION_MARK_SYMBOL, HYPHEN_SYMBOL, OR_SYMBOL,
    PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL, PERCENT_SYMBOL, PLUS_SYMBOL, SLASH_SYMBOL,
    STAR_SYMBOL,
};
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::context::{ParseContext, Parser, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

const UNARY_NEG_FN_NAME: &str = "__neg__";
const UNARY_NOT_FN_NAME: &str = "__not__";
pub(crate) const UNARY_FN_NAMES: &[&str] = &[UNARY_NEG_FN_NAME, UNARY_NOT_FN_NAME];

#[derive(Debug)]
pub(crate) struct Call {
    pub(crate) id: u64,
    pub(crate) scope: Vec<u64>,
    pub(crate) span: Span,
    pub(crate) name: String,
    pub(crate) args: Vec<Expr>,
}

impl NodeRef for &Call {
    fn file_index(&self) -> usize {
        self.span.file_index
    }

    fn id(&self) -> u64 {
        self.id
    }

    // coverage: off (unused because function can be called in itself)
    fn scope(&self) -> &[u64] {
        &self.scope
    }
    // coverage: on
}

impl Call {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
        context.force_parse_any_error();
        let args = context.parse_many(
            |context| Expr::parse(context, parsing::arg_stop_excluded_parser),
            SeparatorParser::MaybeTrailing(|context| {
                Span::parse_symbol(context, COMMA_SYMBOL).map(|_| ())
            }),
            |context| Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ()),
        )?;
        let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
        Ok(Self {
            id: context.next_id(),
            scope: context.scope().to_vec(),
            span: name_span.until(end_span),
            name: context.slice(name_span).into(),
            args,
        })
    }

    pub(crate) fn parse_unary<'context>(
        context: &mut ParseContext<'context>,
        stop_excluded_parser: Parser<'context, ()>,
    ) -> Result<Self, ParseError<'context>> {
        let operator = context.parse_any(&[
            &|context| Span::parse_symbol(context, HYPHEN_SYMBOL),
            &|context| Span::parse_symbol(context, EXCLAMATION_MARK_SYMBOL),
        ])?;
        context.force_parse_any_error();
        let operand = Expr::parse_operand(context, stop_excluded_parser)?;
        let name = match context.slice(operator) {
            symbol if symbol == HYPHEN_SYMBOL.slice => UNARY_NEG_FN_NAME.into(),
            symbol if symbol == EXCLAMATION_MARK_SYMBOL.slice => UNARY_NOT_FN_NAME.into(),
            _ => unreachable!("unrecognized unary operator"),
        };
        Ok(Self {
            id: context.next_id(),
            scope: context.scope().to_vec(),
            span: operator.until(operand.span()),
            name,
            args: vec![operand],
        })
    }

    pub(super) fn from_binary(
        context: &mut ParseContext<'_>,
        left_operand: Expr,
        operator: Span,
        right_operand: Expr,
    ) -> Self {
        let name = match context.slice(operator) {
            symbol if symbol == PLUS_SYMBOL.slice => BINARY_ADD_FN_NAME.into(),
            symbol if symbol == HYPHEN_SYMBOL.slice => BINARY_SUB_FN_NAME.into(),
            symbol if symbol == STAR_SYMBOL.slice => BINARY_MUL_FN_NAME.into(),
            symbol if symbol == SLASH_SYMBOL.slice => BINARY_DIV_FN_NAME.into(),
            symbol if symbol == PERCENT_SYMBOL.slice => BINARY_MOD_FN_NAME.into(),
            symbol if symbol == COMPARE_EQUAL_SYMBOL.slice => BINARY_EQ_FN_NAME.into(),
            symbol if symbol == COMPARE_NOT_EQUAL_SYMBOL.slice => BINARY_NE_FN_NAME.into(),
            symbol if symbol == ANGLE_BRACKET_OPEN_SYMBOL.slice => BINARY_LT_FN_NAME.into(),
            symbol if symbol == COMPARE_LESS_EQUAL_SYMBOL.slice => BINARY_LE_FN_NAME.into(),
            symbol if symbol == ANGLE_BRACKET_CLOSE_SYMBOL.slice => BINARY_GT_FN_NAME.into(),
            symbol if symbol == COMPARE_GREATER_EQUAL_SYMBOL.slice => BINARY_GE_FN_NAME.into(),
            symbol if symbol == AND_SYMBOL.slice => BINARY_AND_FN_NAME.into(),
            symbol if symbol == OR_SYMBOL.slice => BINARY_OR_FN_NAME.into(),
            _ => unreachable!("unrecognized binary operator"),
        };
        Self {
            id: context.next_id(),
            scope: context.scope().to_vec(),
            span: left_operand.span().until(right_operand.span()),
            name,
            args: vec![left_operand, right_operand],
        }
    }

    pub(super) fn from_uniform_syntax(receiver: Expr, call: Self) -> Self {
        let receiver_span = receiver.span();
        let mut args = vec![receiver];
        args.extend(call.args);
        Self {
            id: call.id,
            scope: call.scope,
            span: receiver_span.until(call.span),
            name: call.name,
            args,
        }
    }

    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.args.len())
    }
}
