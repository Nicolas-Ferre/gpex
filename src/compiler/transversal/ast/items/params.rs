use crate::compiler::transversal::ast::exprs::Expr;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct ParamGroup {
    pub(crate) params: Vec<Param>,
    pub(crate) span: Span,
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
}

#[derive(Debug)]
pub(crate) enum ParamQualifier {
    None,
    Const(Span),
}

#[derive(Debug)]
pub(crate) struct ParamRequirement {
    pub(crate) require_span: Span,
    pub(crate) condition: Expr,
}
