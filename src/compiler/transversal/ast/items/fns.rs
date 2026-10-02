use crate::compiler::transversal::ast::exprs::Expr;
use crate::compiler::transversal::ast::items::params::ParamGroup;
use crate::compiler::transversal::ast::statements::Statement;
use crate::utils::parsing::span::Span;

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
    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.params.params.len())
    }

    pub(crate) fn has_requirement(&self) -> bool {
        self.params
            .params
            .iter()
            .any(|param| param.requirement.is_some())
    }
}

#[derive(Debug)]
pub(crate) enum FnBody {
    Intrinsic(Span),
    Statements(FnStatementsBody),
}

impl FnBody {
    pub(crate) fn intrinsic_keyword_span(&self) -> Option<Span> {
        match self {
            Self::Intrinsic(span) => Some(*span),
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
