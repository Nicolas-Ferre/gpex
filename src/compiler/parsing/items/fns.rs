use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::params::ParamGroup;
use crate::compiler::parsing::patterns::IDENT_PATTERN;
use crate::compiler::parsing::statements::{ReturnStatement, Statement};
use crate::compiler::parsing::symbols::{
    ARROW_SYMBOL, BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, CONST_KEYWORD, FN_KEYWORD, PUB_KEYWORD,
};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FnDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) const_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) name: String,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) signature_span: Span,
    #[derive_where(skip)]
    pub(crate) params: ParamGroup,
    #[derive_where(skip)]
    pub(crate) arrow_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) return_type: Option<Expr>,
    #[derive_where(skip)]
    pub(crate) statements: Vec<Statement>,
    #[derive_where(skip)]
    pub(crate) body_span: Span,
    #[derive_where(skip)]
    pub(crate) body_end_span: Span,
}

impl FnDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let scope = context.scope().to_vec();
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            let const_keyword_span = Span::parse_symbol(context, CONST_KEYWORD).ok();
            Span::parse_symbol(context, FN_KEYWORD)?;
            context.force_parse_any_error();
            let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
            let params = ParamGroup::parse(context)?;
            let (arrow_span, return_type, signature_end_span) =
                if let Ok(arrow_span) = Span::parse_symbol(context, ARROW_SYMBOL) {
                    let expr = Expr::parse(context)?;
                    let span = expr.span();
                    (Some(arrow_span), Some(expr), span)
                } else {
                    (None, None, params.span)
                };
            let body_start_span = Span::parse_symbol(context, BRACE_OPEN_SYMBOL)?;
            let statements = context.parse_many(Statement::parse, None, |context| {
                Span::parse_symbol(context, BRACE_CLOSE_SYMBOL).map(|_| ())
            })?;
            let body_end_span = Span::parse_symbol(context, BRACE_CLOSE_SYMBOL)?;
            Ok(Self {
                id,
                scope,
                pub_keyword_span,
                const_keyword_span,
                name_span,
                name: context.slice(name_span).into(),
                signature_span: name_span.until(signature_end_span),
                params,
                arrow_span,
                return_type,
                statements,
                body_span: body_start_span.until(body_end_span),
                body_end_span,
            })
        })
    }

    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.params.params.len())
    }

    pub(crate) fn return_statement(&self) -> Option<&ReturnStatement> {
        self.statements.iter().find_map(|statement| {
            if let Statement::Return(statement) = statement {
                Some(statement)
            } else {
                None
            }
        })
    }
}
