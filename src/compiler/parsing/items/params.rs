use crate::compiler::parsing;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::patterns::IDENT_PATTERN;
use crate::compiler::parsing::symbols::{
    COLON_SYMBOL, COMMA_SYMBOL, CONST_KEYWORD, PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL,
    REQUIRE_KEYWORD,
};
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

#[derive(Debug)]
pub(crate) struct ParamGroup {
    pub(crate) params: Vec<Param>,
    pub(crate) span: Span,
}

impl ParamGroup {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let start_span = Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
        let params = context.parse_many(
            Param::parse,
            SeparatorParser::MaybeTrailing(|context| {
                Span::parse_symbol(context, COMMA_SYMBOL).map(|_| ())
            }),
            |context| Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ()),
        )?;
        let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
        Ok(Self {
            params,
            span: start_span.until(end_span),
        })
    }
}

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Param {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) name: String,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) colon_span: Span,
    #[derive_where(skip)]
    pub(crate) qualifier: ParamQualifier,
    #[derive_where(skip)]
    pub(crate) type_: Expr,
    #[derive_where(skip)]
    pub(crate) requirement: Option<ParamRequirement>,
}

impl Param {
    pub(crate) fn const_mark_span(&self) -> Option<Span> {
        match self.qualifier {
            ParamQualifier::None => None,
            ParamQualifier::Const(span) => Some(span),
        }
    }

    fn parse<'context>(context: &mut ParseContext<'context>) -> Result<Self, ParseError<'context>> {
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        let colon_span = Span::parse_symbol(context, COLON_SYMBOL)?;
        let qualifier = ParamQualifier::parse(context)?;
        let type_ = Expr::parse(context, Self::type_stop_excluded_parser)?;
        let id = context.next_id();
        let requirement = context.parse_any(&[
            &|context| ParamRequirement::parse(context).map(Some),
            &|_| Ok(None),
        ])?;
        Ok(Self {
            id,
            scope: context.scope().to_vec(),
            name: context.slice(name_span).into(),
            name_span,
            colon_span,
            qualifier,
            type_,
            requirement,
        })
    }

    fn type_stop_excluded_parser<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<(), ParseError<'context>> {
        context.parse_any(&[&parsing::arg_stop_excluded_parser, &|context| {
            Span::parse_symbol(context, REQUIRE_KEYWORD).map(|_| ())
        }])
    }
}

#[derive(Debug)]
pub(crate) enum ParamQualifier {
    None,
    Const(Span),
}

impl ParamQualifier {
    fn parse<'context>(context: &mut ParseContext<'context>) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            &|context| Span::parse_symbol(context, CONST_KEYWORD).map(Self::Const),
            &|_| Ok(Self::None),
        ])
    }
}

#[derive(Debug)]
pub(crate) struct ParamRequirement {
    pub(crate) require_span: Span,
    pub(crate) condition: Expr,
}

impl ParamRequirement {
    fn parse<'context>(context: &mut ParseContext<'context>) -> Result<Self, ParseError<'context>> {
        let require_span = Span::parse_symbol(context, REQUIRE_KEYWORD)?;
        context.force_parse_any_error();
        let condition = Expr::parse(context, parsing::arg_stop_excluded_parser)?;
        Ok(Self {
            require_span,
            condition,
        })
    }
}
