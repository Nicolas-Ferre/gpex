use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENT_PATTERN;
use crate::language::symbols::{
    COLON_SYMBOL, COMMA_SYMBOL, PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL,
};
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use std::fmt::Write;

#[derive(Debug)]
pub(crate) struct ParamGroup {
    pub(crate) params: Vec<Param>,
    pub(crate) span: Span,
}

impl ParamGroup {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let start_span = Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
        let params = context.parse_many(
            Param::parse,
            Some(|context| Span::parse_symbol(context, COMMA_SYMBOL).map(|_| ())),
            |context| Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ()),
        )?;
        let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
        Ok(Self {
            params,
            span: start_span.until(end_span),
        })
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        for param in &self.params {
            param.index_refs(indexes);
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        mut dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        for param in &self.params {
            dependencies = param.dependencies(type_, dependencies, indexes)?;
        }
        Ok(dependencies)
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let mut is_param_valid = true;
        for param in &self.params {
            if param.validate(context, indexes).is_err() {
                is_param_valid = false;
            }
        }
        if is_param_valid {
            Ok(())
        } else {
            Err(ValidateError)
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        *shader += "(";
        for param in &self.params {
            param.transpile(shader, indexes);
            *shader += ", ";
        }
        *shader += ")";
    }
}

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct Param {
    pub(crate) id: u64,
    #[derive_where(skip)]
    name: String,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) colon_span: Span,
    #[derive_where(skip)]
    pub(crate) type_: Expr,
}

impl Param {
    fn parse<'context>(context: &mut ParseContext<'context>) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
            let colon_span = Span::parse_symbol(context, COLON_SYMBOL)?;
            let type_ = Expr::parse(context)?;
            Ok(Self {
                id,
                name_span,
                name: context.slice(name_span).into(),
                colon_span,
                type_,
            })
        })
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.type_.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        self.type_.dependencies(type_, dependencies, indexes)
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        if let Some(struct_) = self.type_.const_value(indexes).type_ref() {
            Type::Struct(struct_)
        } else {
            Type::Unknown
        }
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.type_
            .validate(Some(self.colon_span), context, indexes)?;
        validators::expr::check_types(
            self.type_.span(),
            self.type_.type_(indexes),
            None,
            Type::Struct(indexes.search_prelude_type("typeref")),
            context,
        )?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        let id = self.id;
        let type_name = self
            .type_(indexes)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("parameter type should be validated before"))
            .transpile_name();
        _ = write!(shader, "_{id}: {type_name}");
    }
}
