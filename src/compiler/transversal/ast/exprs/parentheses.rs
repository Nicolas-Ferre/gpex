use crate::compiler::transversal::ast::exprs::Expr;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct ParenthesizedExpr {
    pub(crate) span: Span,
    pub(crate) value: Box<Expr>,
}
