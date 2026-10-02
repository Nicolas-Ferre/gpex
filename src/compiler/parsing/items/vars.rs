use crate::compiler::parsing::exprs;
use crate::compiler::transversal::ast::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::transversal::ast::patterns::IDENT_PATTERN;
use crate::compiler::transversal::ast::symbols::{
    CONST_KEYWORD, EQUAL_SYMBOL, PUB_KEYWORD, SEMICOLON_SYMBOL, VAR_KEYWORD,
};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn parse_var_definition<'context>(
    context: &mut ParseContext<'context>,
) -> Result<VarDefinition, ParseError<'context>> {
    let scope = context.scope().to_vec();
    context.define_scope(|context, id| {
        let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
        Span::parse_symbol(context, VAR_KEYWORD)?;
        context.force_parse_any_error();
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        Span::parse_symbol(context, EQUAL_SYMBOL)?;
        let default_value = exprs::parse(context, |context| {
            Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ())
        })?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(VarDefinition {
            id,
            scope,
            pub_keyword_span,
            name_span,
            name: context.slice(name_span).into(),
            default_value,
        })
    })
}

pub(crate) fn parse_const_definition<'context>(
    context: &mut ParseContext<'context>,
) -> Result<ConstDefinition, ParseError<'context>> {
    let scope = context.scope().to_vec();
    context.define_scope(|context, id| {
        let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
        let const_keyword_span = Span::parse_symbol(context, CONST_KEYWORD)?;
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        context.force_parse_any_error();
        Span::parse_symbol(context, EQUAL_SYMBOL)?;
        let value = exprs::parse(context, |context| {
            Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ())
        })?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(ConstDefinition {
            id,
            scope,
            pub_keyword_span,
            const_keyword_span,
            name_span,
            name: context.slice(name_span).into(),
            value,
        })
    })
}
