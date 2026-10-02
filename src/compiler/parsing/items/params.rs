use crate::compiler::parsing;
use crate::compiler::parsing::exprs;
use crate::compiler::transversal::ast::items::params::{
    Param, ParamGroup, ParamQualifier, ParamRequirement,
};
use crate::compiler::transversal::ast::patterns::IDENT_PATTERN;
use crate::compiler::transversal::ast::symbols::{
    COLON_SYMBOL, COMMA_SYMBOL, CONST_KEYWORD, PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL,
    REQUIRE_KEYWORD,
};
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<ParamGroup, ParseError<'context>> {
    let start_span = Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
    let params = context.parse_many(
        parse_param,
        SeparatorParser::MaybeTrailing(|context| {
            Span::parse_symbol(context, COMMA_SYMBOL).map(|_| ())
        }),
        |context| Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ()),
    )?;
    let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
    Ok(ParamGroup {
        params,
        span: start_span.until(end_span),
    })
}

fn parse_param<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Param, ParseError<'context>> {
    let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
    let colon_span = Span::parse_symbol(context, COLON_SYMBOL)?;
    let qualifier = parse_param_qualifier(context)?;
    let type_ = exprs::parse(context, type_stop_excluded_parser)?;
    let id = context.next_id();
    let requirement = context.parse_any(&[
        &|context| parse_param_requirement(context).map(Some),
        &|_| Ok(None),
    ])?;
    Ok(Param {
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

fn parse_param_qualifier<'context>(
    context: &mut ParseContext<'context>,
) -> Result<ParamQualifier, ParseError<'context>> {
    context.parse_any(&[
        &|context| Span::parse_symbol(context, CONST_KEYWORD).map(ParamQualifier::Const),
        &|_| Ok(ParamQualifier::None),
    ])
}

fn parse_param_requirement<'context>(
    context: &mut ParseContext<'context>,
) -> Result<ParamRequirement, ParseError<'context>> {
    let require_span = Span::parse_symbol(context, REQUIRE_KEYWORD)?;
    context.force_parse_any_error();
    let condition = exprs::parse(context, parsing::arg_stop_excluded_parser)?;
    Ok(ParamRequirement {
        require_span,
        condition,
    })
}
