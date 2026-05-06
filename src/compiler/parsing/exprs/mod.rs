pub(crate) mod calls;
pub(crate) mod idents;
pub(crate) mod literals;

use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use calls::Call;
use idents::Ident;
use literals::{BoolLiteral, F32Literal, I32Literal, U32Literal};

#[derive(Debug)]
pub(crate) enum Expr {
    F32Literal(F32Literal),
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    BoolLiteral(BoolLiteral),
    Call(Call),
    Ident(Ident),
}

impl Expr {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| F32Literal::parse(context).map(Self::F32Literal),
            |context| U32Literal::parse(context).map(Self::U32Literal),
            |context| I32Literal::parse(context).map(Self::I32Literal),
            |context| BoolLiteral::parse(context).map(Self::BoolLiteral),
            |context| Call::parse(context).map(Self::Call),
            |context| Call::parse_unary(context).map(Self::Call),
            |context| Ident::parse(context).map(Self::Ident),
        ])
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::F32Literal(node) => node.span,
            Self::U32Literal(node) => node.span,
            Self::I32Literal(node) => node.span,
            Self::BoolLiteral(node) => node.span,
            Self::Call(node) => node.span,
            Self::Ident(node) => node.span,
        }
    }
}
