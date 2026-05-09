use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::patterns::IDENT_PATTERN;
use crate::compiler::parsing::symbols::{
    COMMA_SYMBOL, EXCLAMATION_MARK_SYMBOL, HYPHEN_SYMBOL, PARENTHESIS_CLOSE_SYMBOL,
    PARENTHESIS_OPEN_SYMBOL,
};
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

const UNARY_NEG_FN_NAME: &str = "__neg__";
const UNARY_NOT_FN_NAME: &str = "__not__";
pub(crate) const UNARY_FN_NAMES: &[&str] = &[UNARY_NEG_FN_NAME, UNARY_NOT_FN_NAME];
pub(crate) const OPERATOR_FN_NAME_PREFIX: &str = "__";

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
            Expr::parse,
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
    ) -> Result<Self, ParseError<'context>> {
        let operator = context.parse_any(&[
            |context| Span::parse_symbol(context, HYPHEN_SYMBOL),
            |context| Span::parse_symbol(context, EXCLAMATION_MARK_SYMBOL),
        ])?;
        context.force_parse_any_error();
        let operand = Expr::parse(context)?;
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

    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.args.len())
    }
}
