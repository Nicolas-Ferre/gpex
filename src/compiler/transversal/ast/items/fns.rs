use crate::compiler::transversal::ast::exprs::calls::{UNARY_NEG_FN_NAME, UNARY_NOT_FN_NAME};
use crate::compiler::transversal::ast::exprs::{
    BINARY_ADD_FN_NAME, BINARY_AND_FN_NAME, BINARY_DIV_FN_NAME, BINARY_EQ_FN_NAME,
    BINARY_GE_FN_NAME, BINARY_GT_FN_NAME, BINARY_LE_FN_NAME, BINARY_LT_FN_NAME, BINARY_MOD_FN_NAME,
    BINARY_MUL_FN_NAME, BINARY_NE_FN_NAME, BINARY_OR_FN_NAME, BINARY_SUB_FN_NAME, Expr,
};
use crate::compiler::transversal::ast::items::params::ParamGroup;
use crate::compiler::transversal::ast::statements::Statement;
use crate::compiler::transversal::prelude;
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

    pub(crate) fn intrinsic(&self) -> Option<IntrinsicFn> {
        if !matches!(self.body, FnBody::Intrinsic(_))
            || !prelude::is_prelude_file_index(self.name_span.file_index)
        {
            return None;
        }
        match self.name.as_str() {
            BINARY_ADD_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Add)),
            BINARY_SUB_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Sub)),
            BINARY_MUL_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Mul)),
            BINARY_DIV_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Div)),
            BINARY_MOD_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Mod)),
            BINARY_EQ_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Eq)),
            BINARY_NE_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Ne)),
            BINARY_LT_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Lt)),
            BINARY_LE_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Le)),
            BINARY_GT_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Gt)),
            BINARY_GE_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Ge)),
            BINARY_AND_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::And)),
            BINARY_OR_FN_NAME => Some(IntrinsicFn::Binary(BinaryIntrinsicFn::Or)),
            UNARY_NEG_FN_NAME => Some(IntrinsicFn::Unary(UnaryIntrinsicFn::Neg)),
            UNARY_NOT_FN_NAME => Some(IntrinsicFn::Unary(UnaryIntrinsicFn::Not)),
            "mul_add" => Some(IntrinsicFn::MulAdd),
            "typeof" => Some(IntrinsicFn::Typeof),
            "sizeof" => Some(IntrinsicFn::Sizeof),
            _ => unreachable!("unknown prelude intrinsic function name `{}`", self.name),
        }
    }

    pub(crate) fn has_requirement(&self) -> bool {
        self.params
            .params
            .iter()
            .any(|param| param.requirement.is_some())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntrinsicFn {
    Binary(BinaryIntrinsicFn),
    Unary(UnaryIntrinsicFn),
    MulAdd,
    Typeof,
    Sizeof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryIntrinsicFn {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinaryIntrinsicFn {
    pub(crate) fn is_logical_operator(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    pub(crate) fn is_comparison_operator(self) -> bool {
        matches!(
            self,
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryIntrinsicFn {
    Neg,
    Not,
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
