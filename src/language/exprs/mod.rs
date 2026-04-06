pub(crate) mod fn_call;
pub(crate) mod ident;
pub(crate) mod literals;

use crate::compiler::consts::ConstValue;
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::fn_call::FnCall;
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
    FunctionCall(FnCall),
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
            |context| FnCall::parse(context).map(Self::FunctionCall),
            |context| Ident::parse(context).map(Self::Identifier),
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
        if type_ == DependencyType::Transpilation
            && self.const_value(indexes) != ConstValue::RuntimeValue
        {
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

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        match self {
            Self::F32Literal(_) => Type::Struct(F32Literal::type_(indexes)),
            Self::U32Literal(_) => Type::Struct(U32Literal::type_(indexes)),
            Self::I32Literal(_) => Type::Struct(I32Literal::type_(indexes)),
            Self::BoolLiteral(_) => Type::Struct(BoolLiteral::type_(indexes)),
            Self::FunctionCall(node) => node.type_(indexes),
            Self::Identifier(node) => node.type_(indexes),
        }
    }

    pub(crate) fn const_value<'index>(&self, indexes: &Indexes<'index>) -> ConstValue<'index> {
        match self {
            Self::F32Literal(node) => node.const_value(),
            Self::U32Literal(node) => node.const_value(),
            Self::I32Literal(node) => node.const_value(),
            Self::BoolLiteral(node) => node.const_value(),
            Self::FunctionCall(node) => node.const_value(indexes),
            Self::Identifier(node) => node.const_value(indexes),
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
        const_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::F32Literal(node) => node.validate(context),
            Self::U32Literal(node) => node.validate(context),
            Self::I32Literal(node) => node.validate(context),
            Self::BoolLiteral(_) => Ok(()),
            Self::FunctionCall(node) => {
                validators::expr::check_no_return_type(node, node.span, context, indexes)?;
                node.validate(const_mark_span, context, indexes)
            }
            Self::Identifier(node) => node.validate(const_mark_span, context, indexes),
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        let value = self.const_value(indexes);
        if value == ConstValue::RuntimeValue {
            match self {
                Self::FunctionCall(node) => node.transpile(shader, indexes),
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
