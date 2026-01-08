use crate::compiler::constants::Constant;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::expressions::literals::{I32Literal, U32Literal};
use crate::language::items::struct_::StructDefinition;
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use identifier::Identifier;

pub(crate) mod identifier;
pub(crate) mod literals;

#[derive(Debug)]
pub(crate) enum Expression {
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    Identifier(Identifier),
}

impl Expression {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| U32Literal::parse(context).map(Self::U32Literal),
            |context| I32Literal::parse(context).map(Self::I32Literal),
            |context| Identifier::parse(context).map(Self::Identifier),
        ])
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::U32Literal(_) | Self::I32Literal(_) => (),
            Self::Identifier(node) => node.index(indexes),
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        match self {
            Self::U32Literal(_) | Self::I32Literal(_) => Ok(dependencies),
            Self::Identifier(node) => node.dependencies(dependencies, indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::U32Literal(node) => node.validate(context),
            Self::I32Literal(node) => node.validate(context),
            Self::Identifier(node) => node.validate(constant_mark_span, context, indexes),
        }
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> &'index StructDefinition {
        match self {
            Self::U32Literal(_) => U32Literal::type_(indexes),
            Self::I32Literal(_) => I32Literal::type_(indexes),
            Self::Identifier(node) => node.type_(indexes),
        }
    }

    pub(crate) fn constant<'index>(&self, indexes: &Indexes<'index>) -> Option<Constant<'index>> {
        match self {
            Self::U32Literal(node) => Some(node.constant()),
            Self::I32Literal(node) => Some(node.constant()),
            Self::Identifier(node) => node.constant(indexes),
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match self {
            Self::U32Literal(node) => node.transpile(shader),
            Self::I32Literal(node) => node.transpile(shader),
            Self::Identifier(node) => node.transpile(shader, indexes),
        }
    }
}
