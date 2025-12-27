use crate::compiler::constants::ConstValue;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
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
        ctx.parse_any(&[
            |ctx| I32Lit::parse(ctx).map(Self::I32Lit),
            |ctx| IdentExpr::parse(ctx).map(Self::Ident),
        ])
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Ident(node) => node.pre_validate(indexes),
            Self::I32Lit(_) => (),
        }
    }

    pub(crate) fn dependencies<'a>(
        &self,
        dependencies: Dependencies<'a>,
        indexes: &Indexes<'a>,
    ) -> Result<Dependencies<'a>, Vec<Span>> {
        match self {
            Self::I32Lit(_) => Ok(dependencies),
            Self::Ident(node) => node.dependencies(dependencies, indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        const_span: Option<&Span>,
        ctx: &mut ValidateCtx<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::I32Lit(node) => node.validate(ctx, indexes),
            Self::Ident(node) => node.validate(const_span, ctx, indexes),
        }
    }

    pub(crate) fn const_value(&self, indexes: &Indexes<'_>) -> Option<ConstValue> {
        match self {
            Self::I32Lit(node) => Some(node.const_value(indexes).clone()),
            Self::Ident(node) => node.const_value(indexes),
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match self {
            Self::I32Lit(node) => node.transpile(shader, indexes),
            Self::Ident(node) => node.transpile(shader, indexes),
        }
    }
}
