pub(crate) mod call;
pub(crate) mod ident;
pub(crate) mod literals;

use crate::compiler::consts::{ConstContext, ConstValue};
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::call::Call;
use crate::language::exprs::literals::{BoolLiteral, F32Literal, I32Literal, U32Literal};
use crate::language::items::ItemRef;
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use ident::Ident;

#[derive(Debug)]
pub(crate) enum Expr {
    F32Literal(F32Literal),
    U32Literal(U32Literal),
    I32Literal(I32Literal),
    BoolLiteral(BoolLiteral),
    Call(Call),
    Identifier(Ident),
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
            |context| Ident::parse(context).map(Self::Identifier),
        ])
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Self::F32Literal(node) => node.span,
            Self::U32Literal(node) => node.span,
            Self::I32Literal(node) => node.span,
            Self::BoolLiteral(node) => node.span,
            Self::Call(node) => node.span,
            Self::Identifier(node) => node.span,
        }
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::F32Literal(_)
            | Self::U32Literal(_)
            | Self::I32Literal(_)
            | Self::BoolLiteral(_) => (),
            Self::Call(node) => node.index(indexes),
            Self::Identifier(node) => node.index(indexes),
        }
    }

    fn is_const(&self, indexes: &Indexes<'_>) -> bool {
        match self {
            Self::F32Literal(_)
            | Self::U32Literal(_)
            | Self::I32Literal(_)
            | Self::BoolLiteral(_) => true,
            Self::Call(node) => node.is_const(indexes),
            Self::Identifier(node) => node.is_const(indexes),
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        if type_ == DependencyType::Transpilation && self.is_const(indexes) {
            Ok(dependencies)
        } else {
            match self {
                Self::F32Literal(_)
                | Self::U32Literal(_)
                | Self::I32Literal(_)
                | Self::BoolLiteral(_) => Ok(dependencies),
                Self::Call(node) => node.dependencies(type_, dependencies, indexes),
                Self::Identifier(node) => node.dependencies(type_, dependencies, indexes),
            }
        }
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        match self {
            Self::F32Literal(_) => Type::Struct(F32Literal::type_(indexes)),
            Self::U32Literal(_) => Type::Struct(U32Literal::type_(indexes)),
            Self::I32Literal(_) => Type::Struct(I32Literal::type_(indexes)),
            Self::BoolLiteral(_) => Type::Struct(BoolLiteral::type_(indexes)),
            Self::Call(node) => node.type_(indexes),
            Self::Identifier(node) => node.type_(indexes),
        }
    }

    pub(crate) fn const_value<'index>(
        &self,
        indexes: &Indexes<'index>,
        context: &mut ConstContext<'index>,
    ) -> ConstValue<'index> {
        match self {
            Self::F32Literal(node) => node.const_value(),
            Self::U32Literal(node) => node.const_value(),
            Self::I32Literal(node) => node.const_value(),
            Self::BoolLiteral(node) => node.const_value(),
            Self::Call(node) => node.const_value(indexes, context),
            Self::Identifier(node) => node.const_value(indexes, context),
        }
    }

    pub(crate) fn is_ref(&self, indexes: &Indexes<'_>) -> Option<bool> {
        match self {
            Self::F32Literal(_)
            | Self::U32Literal(_)
            | Self::I32Literal(_)
            | Self::BoolLiteral(_)
            | Self::Call(_) => Some(false),
            Self::Identifier(node) => node.is_ref(indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        const_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::F32Literal(node) => node.validate(context),
            Self::U32Literal(node) => node.validate(context),
            Self::I32Literal(node) => node.validate(context),
            Self::BoolLiteral(_) => Ok(()),
            Self::Call(node) => {
                validators::expr::check_no_return_type(node, node.span, context, indexes)?;
                node.validate(const_mark_span, context, indexes)
            }
            Self::Identifier(node) => node.validate(const_mark_span, context, indexes),
        }
    }

    pub(crate) fn transpile<'index>(
        &self,
        shader: &mut String,
        indexes: &Indexes<'index>,
        context: &mut ConstContext<'index>,
    ) {
        let value = self.const_value(indexes, context);
        if value == ConstValue::RuntimeValue {
            match self {
                Self::Call(node) => node.transpile(shader, indexes, context),
                Self::Identifier(node) => node.transpile(shader, indexes),
                Self::F32Literal(_)
                | Self::U32Literal(_)
                | Self::I32Literal(_)
                | Self::BoolLiteral(_) => unreachable!("literals should be constant"),
            }
        } else {
            value.transpile(shader);
        }
    }
}
