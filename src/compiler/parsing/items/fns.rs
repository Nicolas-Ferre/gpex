use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::params::ParamGroup;
use crate::compiler::parsing::patterns::IDENT_PATTERN;
use crate::compiler::parsing::statements::Statement;
use crate::compiler::parsing::symbols::{
    ARROW_SYMBOL, BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, COMPILERIMPL_KEYWORD, CONST_KEYWORD,
    EQUAL_SYMBOL, FN_KEYWORD, PUB_KEYWORD, SEMICOLON_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
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
    pub(crate) signature_span_with_return: Span,
    #[derive_where(skip)]
    pub(crate) signature_span_without_return: Span,
    #[derive_where(skip)]
    pub(crate) params: ParamGroup,
    #[derive_where(skip)]
    pub(crate) arrow_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) return_type: Option<Expr>,
    #[derive_where(skip)]
    pub(crate) body: FnBody,
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
                    let expr = Expr::parse(context, Self::parse_return_type_stop)?;
                    let span = expr.span();
                    (Some(arrow_span), Some(expr), span)
                } else {
                    (None, None, params.span)
                };
            let body =
                context.parse_any(&[&Self::parse_body_statements, &Self::parse_compilerimpl])?;
            Ok(Self {
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

    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.params.params.len())
    }

    pub(crate) fn compilerimpl(&self) -> Option<CompilerImpl> {
        if !matches!(self.body, FnBody::Compilerimpl(_)) {
            return None;
        }
        match self.name.as_str() {
            "__add__" => Some(CompilerImpl::Add),
            "typeof" => Some(CompilerImpl::Typeof),
            "sizeof" => Some(CompilerImpl::Sizeof),
            _ => None,
        }
    }

    pub(crate) fn has_requirement(&self) -> bool {
        self.params
            .params
            .iter()
            .any(|param| param.requirement.is_some())
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
        let statements =
            context.parse_many(Statement::parse, SeparatorParser::None, |context| {
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

    fn parse_compilerimpl<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<FnBody, ParseError<'context>> {
        Span::parse_symbol(context, EQUAL_SYMBOL)?;
        let compilerimpl_keyword_span = Span::parse_symbol(context, COMPILERIMPL_KEYWORD)?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(FnBody::Compilerimpl(compilerimpl_keyword_span))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompilerImpl {
    Add,
    Typeof,
    Sizeof,
}

#[derive(Debug)]
pub(crate) enum FnBody {
    Compilerimpl(Span),
    Statements(FnStatementsBody),
}

impl FnBody {
    pub(crate) fn compilerimpl_keyword_span(&self) -> Option<Span> {
        match self {
            Self::Compilerimpl(span) => Some(*span),
            Self::Statements(_) => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FnStatementsBody {
    pub(crate) statements: Vec<Statement>,
    pub(crate) body_span: Span,
    pub(crate) body_start_span: Span,
    pub(crate) body_end_span: Span,
}
