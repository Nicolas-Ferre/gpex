use crate::compiler::parsing::exprs;
use crate::compiler::parsing::items::params;
use crate::compiler::parsing::statements;
use crate::compiler::transversal::ast::exprs::Expr;
use crate::compiler::transversal::ast::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::transversal::ast::items::params::ParamGroup;
use crate::compiler::transversal::ast::patterns::IDENT_PATTERN;
use crate::compiler::transversal::ast::symbols::{
    ARROW_SYMBOL, BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, CONST_KEYWORD, EQUAL_SYMBOL, FN_KEYWORD,
    INTRINSIC_KEYWORD, PUB_KEYWORD, SEMICOLON_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<FnDefinition, ParseError<'context>> {
    let scope = context.scope().to_vec();
    context.define_scope(|context, id| {
        let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
        let const_keyword_span = Span::parse_symbol(context, CONST_KEYWORD).ok();
        Span::parse_symbol(context, FN_KEYWORD)?;
        context.force_parse_any_error();
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        let params = params::parse(context)?;
        let (arrow_span, return_type, signature_end_span) = parse_return_type(context, &params)?;
        let body = context.parse_any(&[&parse_body_statements, &parse_intrinsic])?;
        Ok(FnDefinition {
            id,
            scope,
            pub_keyword_span,
            const_keyword_span,
            name_span,
            name: context.slice(name_span).into(),
            signature_span_with_return: name_span.until(signature_end_span),
            signature_span_without_return: name_span.until(params.span),
            params,
            arrow_span,
            return_type,
            body,
        })
    })
}

fn parse_return_type<'context>(
    context: &mut ParseContext<'context>,
    params: &ParamGroup,
) -> Result<(Option<Span>, Option<Expr>, Span), ParseError<'context>> {
    let Ok(arrow_span) = Span::parse_symbol(context, ARROW_SYMBOL) else {
        return Ok((None, None, params.span));
    };
    let expr = exprs::parse(context, parse_return_type_stop)?;
    let span = expr.span();
    Ok((Some(arrow_span), Some(expr), span))
}

fn parse_return_type_stop<'context>(
    context: &mut ParseContext<'context>,
) -> Result<(), ParseError<'context>> {
    context.parse_any(&[
        &|context| Span::parse_symbol(context, BRACE_OPEN_SYMBOL).map(|_| ()),
        &|context| Span::parse_symbol(context, EQUAL_SYMBOL).map(|_| ()),
    ])
}

fn parse_body_statements<'context>(
    context: &mut ParseContext<'context>,
) -> Result<FnBody, ParseError<'context>> {
    let body_start_span = Span::parse_symbol(context, BRACE_OPEN_SYMBOL)?;
    let statements = context.parse_many(statements::parse, SeparatorParser::None, |context| {
        Span::parse_symbol(context, BRACE_CLOSE_SYMBOL).map(|_| ())
    })?;
    let body_end_span = Span::parse_symbol(context, BRACE_CLOSE_SYMBOL)?;
    Ok(FnBody::Statements(FnStatementsBody {
        statements,
        body_span: body_start_span.until(body_end_span),
        body_start_span,
        body_end_span,
    }))
}

fn parse_intrinsic<'context>(
    context: &mut ParseContext<'context>,
) -> Result<FnBody, ParseError<'context>> {
    Span::parse_symbol(context, EQUAL_SYMBOL)?;
    let intrinsic_keyword_span = Span::parse_symbol(context, INTRINSIC_KEYWORD)?;
    Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
    Ok(FnBody::Intrinsic(intrinsic_keyword_span))
}
