pub(crate) mod calls;
pub(crate) mod idents;
pub(crate) mod literals;
pub(crate) mod parentheses;

use crate::utils::parsing::span::Span;
use calls::Call;
use idents::Ident;
use literals::{BoolLiteral, F32Literal, I32Literal, U32Literal};
use parentheses::ParenthesizedExpr;

pub(crate) const BINARY_ADD_FN_NAME: &str = "__add__";
pub(crate) const BINARY_SUB_FN_NAME: &str = "__sub__";
pub(crate) const BINARY_MUL_FN_NAME: &str = "__mul__";
pub(crate) const BINARY_DIV_FN_NAME: &str = "__div__";
pub(crate) const BINARY_MOD_FN_NAME: &str = "__mod__";
pub(crate) const BINARY_EQ_FN_NAME: &str = "__eq__";
pub(crate) const BINARY_NE_FN_NAME: &str = "__ne__";
pub(crate) const BINARY_LT_FN_NAME: &str = "__lt__";
pub(crate) const BINARY_LE_FN_NAME: &str = "__le__";
pub(crate) const BINARY_GT_FN_NAME: &str = "__gt__";
pub(crate) const BINARY_GE_FN_NAME: &str = "__ge__";
pub(crate) const BINARY_AND_FN_NAME: &str = "__and__";
pub(crate) const BINARY_OR_FN_NAME: &str = "__or__";
pub(crate) const BINARY_FN_NAMES: &[&str] = &[
    BINARY_ADD_FN_NAME,
    BINARY_SUB_FN_NAME,
    BINARY_MUL_FN_NAME,
    BINARY_DIV_FN_NAME,
    BINARY_MOD_FN_NAME,
    BINARY_EQ_FN_NAME,
    BINARY_NE_FN_NAME,
    BINARY_LT_FN_NAME,
    BINARY_LE_FN_NAME,
    BINARY_GT_FN_NAME,
    BINARY_GE_FN_NAME,
    BINARY_AND_FN_NAME,
    BINARY_OR_FN_NAME,
];

#[derive(Debug)]
pub(crate) enum Expr {
    F32Literal(F32Literal),
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    BoolLiteral(BoolLiteral),
    Wildcard(Span),
    Call(Call),
    Ident(Ident),
    Parenthesized(ParenthesizedExpr),
}

impl Expr {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::F32Literal(literal) => literal.span,
            Self::U32Literal(literal) => literal.span,
            Self::I32Literal(literal) => literal.span,
            Self::BoolLiteral(literal) => literal.span,
            Self::Wildcard(span) => *span,
            Self::Call(call) => call.span,
            Self::Ident(ident) => ident.span,
            Self::Parenthesized(parenthesized) => parenthesized.span,
        }
    }

    pub(crate) fn unparenthesized(&self) -> &Self {
        match self {
            Self::Parenthesized(parenthesized) => parenthesized.value.unparenthesized(),
            Self::F32Literal(_)
            | Self::U32Literal(_)
            | Self::I32Literal(_)
            | Self::BoolLiteral(_)
            | Self::Wildcard(_)
            | Self::Call(_)
            | Self::Ident(_) => self,
        }
    }
}

pub(crate) fn is_operator_fn_name(name: &str) -> bool {
    BINARY_FN_NAMES.contains(&name) || calls::UNARY_FN_NAMES.contains(&name)
}
