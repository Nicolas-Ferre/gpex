use crate::compiler::transversal::ast::exprs::Expr;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) enum Statement {
    Return(ReturnStatement),
    Assignment(AssignmentStatement),
}

impl Statement {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Return(statement) => statement.span,
            Self::Assignment(statement) => statement.span,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReturnStatement {
    pub(crate) span: Span,
    pub(crate) value: Expr,
}

#[derive(Debug)]
pub(crate) struct AssignmentStatement {
    pub(crate) span: Span,
    pub(crate) assigned: Expr,
    pub(crate) value: Expr,
}
