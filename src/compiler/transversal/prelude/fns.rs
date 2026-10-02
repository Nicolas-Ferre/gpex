use crate::compiler::transversal::ast::exprs::calls::{UNARY_NEG_FN_NAME, UNARY_NOT_FN_NAME};
use crate::compiler::transversal::ast::exprs::{
    BINARY_ADD_FN_NAME, BINARY_AND_FN_NAME, BINARY_DIV_FN_NAME, BINARY_EQ_FN_NAME,
    BINARY_GE_FN_NAME, BINARY_GT_FN_NAME, BINARY_LE_FN_NAME, BINARY_LT_FN_NAME, BINARY_MOD_FN_NAME,
    BINARY_MUL_FN_NAME, BINARY_NE_FN_NAME, BINARY_OR_FN_NAME, BINARY_SUB_FN_NAME,
};
use crate::compiler::transversal::ast::items::ItemRef;
use crate::compiler::transversal::ast::items::fns::{FnBody, FnDefinition};
use crate::compiler::transversal::prelude::files;

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

pub(crate) fn is_typeof(item: ItemRef<'_>) -> bool {
    matches!(item, ItemRef::Fn(fn_) if intrinsic(fn_) == Some(IntrinsicFn::Typeof))
}

pub(crate) fn intrinsic(fn_: &FnDefinition) -> Option<IntrinsicFn> {
    if !matches!(fn_.body, FnBody::Intrinsic(_))
        || !files::is_prelude_file_index(fn_.name_span.file_index)
    {
        return None;
    }
    match fn_.name.as_str() {
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
        _ => unreachable!("unknown prelude intrinsic function name `{}`", fn_.name),
    }
}
