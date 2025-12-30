use crate::compiler::constants::Constant;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::expressions::literals::I32Literal;
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use identifier::Identifier;

pub(crate) mod identifier;
pub(crate) mod literals;

#[derive(Debug)]
pub(crate) enum Expression {
    I32Literal(I32Literal),
    Identifier(Identifier),
}

impl Expression {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| I32Literal::parse(context).map(Self::I32Literal),
            |context| Identifier::parse(context).map(Self::Identifier),
        ])
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Identifier(node) => node.index(indexes),
            Self::I32Literal(_) => (),
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        match self {
            Self::I32Literal(_) => Ok(dependencies),
            Self::Identifier(node) => node.dependencies(dependencies, indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::I32Literal(node) => node.validate(context, indexes),
            Self::Identifier(node) => node.validate(constant_mark_span, context, indexes),
        }
    }

    pub(crate) fn constant(&self, indexes: &Indexes<'_>) -> Option<Constant> {
        match self {
            Self::I32Literal(node) => Some(node.constant(indexes).clone()),
            Self::Identifier(node) => node.constant(indexes),
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match self {
            Self::I32Literal(node) => node.transpile(shader, indexes),
            Self::Identifier(node) => node.transpile(shader, indexes),
        }
    }
}
