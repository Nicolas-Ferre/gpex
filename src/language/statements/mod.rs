pub(crate) mod assignment;
pub(crate) mod return_;

use crate::compiler::consts::ConstContext;
use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::items::ItemRef;
use crate::language::statements::assignment::AssignmentStatement;
use crate::language::statements::return_::ReturnStatement;
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use crate::utils::validation::{ValidateContext, ValidateError};

#[derive(Debug)]
pub(crate) enum Statement {
    Return(ReturnStatement),
    Assignment(AssignmentStatement),
}

impl Statement {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| ReturnStatement::parse(context).map(Self::Return),
            |context| AssignmentStatement::parse(context).map(Self::Assignment),
        ])
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Return(node) => node.index(indexes),
            Self::Assignment(node) => node.index(indexes),
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        match self {
            Self::Return(node) => node.dependencies(type_, dependencies, indexes),
            Self::Assignment(node) => node.dependencies(type_, dependencies, indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        const_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::Return(node) => node.validate(const_mark_span, context, indexes),
            Self::Assignment(node) => node.validate(const_mark_span, context, indexes),
        }
    }

    pub(crate) fn transpile<'index>(
        &self,
        shader: &mut String,
        indexes: &Indexes<'index>,
        context: &mut ConstContext<'index>,
    ) {
        match self {
            Self::Return(node) => node.transpile(shader, indexes, context),
            Self::Assignment(node) => node.transpile(shader, indexes, context),
        }
    }
}
