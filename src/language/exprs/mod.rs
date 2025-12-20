use crate::compiler::indexes::Indexes;
use crate::compiletools::parsing::{ParseCtx, ParseError};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::exprs::literals::I32Lit;
use ident::IdentExpr;

pub(crate) mod ident;
pub(crate) mod literals;

#[derive(Debug)]
pub(crate) enum Expr {
    I32Lit(I32Lit),
    Ident(IdentExpr),
}

impl Expr {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        Err(ParseError::merge([
            match I32Lit::parse(ctx) {
                Ok(node) => return Ok(Self::I32Lit(node)),
                Err(err) => err,
            },
            match IdentExpr::parse(ctx) {
                Ok(node) => return Ok(Self::Ident(node)),
                Err(err) => err,
            },
        ]))
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Ident(node) => node.pre_validate(indexes),
            Self::I32Lit(_) => (),
        }
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::Ident(node) => node.validate(ctx, indexes)?,
            Self::I32Lit(node) => node.validate(ctx),
        }
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match self {
            Self::I32Lit(node) => node.transpile(shader),
            Self::Ident(node) => node.transpile(shader, indexes),
        }
    }
}
