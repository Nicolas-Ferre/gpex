use crate::compiler::transversal::ast::items::types::StructDefinition;
use crate::compiler::transversal::ast::patterns::IDENT_PATTERN;
use crate::compiler::transversal::ast::symbols::{
    BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, EQUAL_SYMBOL, INTRINSIC_KEYWORD, PUB_KEYWORD,
    STRUCT_KEYWORD,
};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<StructDefinition, ParseError<'context>> {
    let scope = context.scope().to_vec();
    context.define_scope(move |context, id| {
        let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
        Span::parse_symbol(context, STRUCT_KEYWORD)?;
        context.force_parse_any_error();
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        Span::parse_symbol(context, EQUAL_SYMBOL)?;
        let intrinsic_keyword_span = Span::parse_symbol(context, INTRINSIC_KEYWORD)?;
        Span::parse_symbol(context, BRACE_OPEN_SYMBOL)?;
        Span::parse_symbol(context, BRACE_CLOSE_SYMBOL)?;
        Ok(StructDefinition {
            id,
            scope,
            pub_keyword_span,
            name_span,
            name: context.slice(name_span).into(),
            intrinsic_keyword_span,
        })
    })
}
