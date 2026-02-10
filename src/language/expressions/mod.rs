use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::expressions::function_call::FunctionCall;
use crate::language::expressions::literals::{BoolLiteral, F32Literal, I32Literal, U32Literal};
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use identifier::Identifier;

pub(crate) mod function_call;
pub(crate) mod identifier;
pub(crate) mod literals;

#[derive(Debug)]
pub(crate) enum Expression {
    F32Literal(F32Literal),
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    BoolLiteral(BoolLiteral),
    FunctionCall(FunctionCall),
    Identifier(Identifier),
}

impl Expression {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| F32Literal::parse(context).map(Self::F32Literal),
            |context| U32Literal::parse(context).map(Self::U32Literal),
            |context| I32Literal::parse(context).map(Self::I32Literal),
            |context| BoolLiteral::parse(context).map(Self::BoolLiteral),
            |context| FunctionCall::parse(context).map(Self::FunctionCall),
            |context| Identifier::parse(context).map(Self::Identifier),
        ])
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::F32Literal(node) => node.span,
            Self::U32Literal(node) => node.span,
            Self::I32Literal(node) => node.span,
            Self::BoolLiteral(node) => node.span,
            Self::FunctionCall(node) => node.span,
            Self::Identifier(node) => node.span,
        }
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::F32Literal(_)
            | Self::U32Literal(_)
            | Self::I32Literal(_)
            | Self::BoolLiteral(_) => (),
            Self::FunctionCall(node) => node.index(indexes),
            Self::Identifier(node) => node.index(indexes),
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        if type_ == DependencyType::Transpilation && self.constant(indexes).is_some() {
            Ok(dependencies)
        } else {
            match self {
                Self::F32Literal(_)
                | Self::U32Literal(_)
                | Self::I32Literal(_)
                | Self::BoolLiteral(_) => Ok(dependencies),
                Self::FunctionCall(node) => node.dependencies(type_, dependencies, indexes),
                Self::Identifier(node) => node.dependencies(type_, dependencies, indexes),
            }
        }
    }

    pub(crate) fn type_<'index>(
        &self,
        indexes: &Indexes<'index>,
    ) -> Option<&'index StructDefinition> {
        match self {
            Self::F32Literal(_) => Some(F32Literal::type_(indexes)),
            Self::U32Literal(_) => Some(U32Literal::type_(indexes)),
            Self::I32Literal(_) => Some(I32Literal::type_(indexes)),
            Self::BoolLiteral(_) => Some(BoolLiteral::type_(indexes)),
            Self::FunctionCall(node) => node.type_(indexes),
            Self::Identifier(node) => node.type_(indexes),
        }
    }

    pub(crate) fn constant<'index>(&self, indexes: &Indexes<'index>) -> Option<Constant<'index>> {
        match self {
            Self::F32Literal(node) => Some(node.constant()),
            Self::U32Literal(node) => Some(node.constant()),
            Self::I32Literal(node) => Some(node.constant()),
            Self::BoolLiteral(node) => Some(node.constant()),
            Self::FunctionCall(node) => node.constant(indexes),
            Self::Identifier(node) => node.constant(indexes),
        }
    }

    pub(crate) fn is_ref(&self, indexes: &Indexes<'_>) -> Option<bool> {
        match self {
            Self::F32Literal(_)
            | Self::U32Literal(_)
            | Self::I32Literal(_)
            | Self::BoolLiteral(_)
            | Self::FunctionCall(_) => Some(false),
            Self::Identifier(node) => node.is_ref(indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::F32Literal(node) => node.validate(context),
            Self::U32Literal(node) => node.validate(context),
            Self::I32Literal(node) => node.validate(context),
            Self::BoolLiteral(_) => Ok(()),
            Self::FunctionCall(node) => node.validate(constant_mark_span, context, indexes),
            Self::Identifier(node) => node.validate(constant_mark_span, context, indexes),
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        if let Some(constant) = self.constant(indexes) {
            constant.transpile(shader);
        } else {
            match self {
                Self::FunctionCall(node) => node.transpile(shader, indexes),
                Self::Identifier(node) => node.transpile(shader, indexes),
                Self::F32Literal(_)
                | Self::U32Literal(_)
                | Self::I32Literal(_)
                | Self::BoolLiteral(_) => unreachable!("literals should be constant"),
            }
        }
    }
}
