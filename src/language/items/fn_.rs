use crate::compiler::consts::ConstValue;
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::language::items::param::ParamGroup;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::IDENT_PATTERN;
use crate::language::statements::Statement;
use crate::language::statements::return_::ReturnStatement;
use crate::language::symbols::{
    ARROW_SYMBOL, BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, CONST_KEYWORD, FN_KEYWORD, PUB_KEYWORD,
};
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use crate::validators::ident::Case;
use itertools::Itertools;
use std::fmt::Write;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct FnDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) const_keyword_span: Option<Span>,
    #[derive_where(skip)]
    name: String,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) signature_span: Span,
    #[derive_where(skip)]
    pub(crate) params: ParamGroup,
    #[derive_where(skip)]
    arrow_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) return_type: Option<Expr>,
    #[derive_where(skip)]
    statements: Vec<Statement>,
    #[derive_where(skip)]
    body_span: Span,
    #[derive_where(skip)]
    body_end_span: Span,
}

impl FnDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            let const_keyword_span = Span::parse_symbol(context, CONST_KEYWORD).ok();
            Span::parse_symbol(context, FN_KEYWORD)?;
            context.force_parse_any_error();
            let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
            let params = ParamGroup::parse(context)?;
            let (arrow_span, return_type, signature_end_span) =
                if let Ok(arrow_span) = Span::parse_symbol(context, ARROW_SYMBOL) {
                    let expr = Expr::parse(context)?;
                    let span = expr.span();
                    (Some(arrow_span), Some(expr), span)
                } else {
                    (None, None, params.span)
                };
            let body_start_span = Span::parse_symbol(context, BRACE_OPEN_SYMBOL)?;
            let statements = context.parse_many(Statement::parse, None, |context| {
                Span::parse_symbol(context, BRACE_CLOSE_SYMBOL).map(|_| ())
            })?;
            let body_end_span = Span::parse_symbol(context, BRACE_CLOSE_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                pub_keyword_span,
                const_keyword_span,
                name_span,
                name: context.slice(name_span).into(),
                signature_span: name_span.until(signature_end_span),
                params,
                arrow_span,
                return_type,
                statements,
                body_span: body_start_span.until(body_end_span),
                body_end_span,
            })
        })
    }

    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.params.params.len())
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(ItemRef::Fn(self));
    }

    pub(crate) fn index_signatures(&self, indexes: &mut Indexes<'_>) {
        self.params.index_refs(indexes);
        if let Some(return_type) = &self.return_type {
            return_type.index(indexes);
        }
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        for statement in &self.statements {
            statement.index(indexes);
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        mut dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        dependencies = self.params.dependencies(type_, dependencies, indexes)?;
        if let Some(return_type) = &self.return_type {
            dependencies = return_type.dependencies(type_, dependencies, indexes)?;
        }
        for statement in &self.statements {
            dependencies = statement.dependencies(type_, dependencies, indexes)?;
        }
        Ok(dependencies)
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        let Some(return_type) = self.return_type.as_ref() else {
            return Type::NoReturn;
        };
        let value = return_type.const_value(indexes);
        if let Some(struct_) = value.type_ref() {
            Type::Struct(struct_)
        } else {
            Type::Unknown
        }
    }

    pub(crate) fn displayed_key(&self, indexes: &Indexes<'_>) -> String {
        let fn_name = &self.name;
        let param_types = self
            .params
            .params
            .iter()
            .map(|param| param.type_(indexes).name())
            .join(", ");
        format!("{fn_name}({param_types})")
    }

    fn return_statement(&self) -> Option<&ReturnStatement> {
        self.statements.iter().find_map(|statement| {
            if let Statement::Return(statement) = statement {
                Some(statement)
            } else {
                None
            }
        })
    }

    pub(crate) fn const_value<'index>(&self, indexes: &Indexes<'index>) -> ConstValue<'index> {
        if self.const_keyword_span.is_none() {
            ConstValue::RuntimeValue
        } else if let Some(return_) = self.return_statement() {
            return_.value.const_value(indexes)
        } else {
            ConstValue::Unknown
        }
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Fn(self);
        let dependencies =
            self.dependencies(DependencyType::CycleDetection, Dependencies::new(), indexes);
        validators::item::check_circular_dependencies(ref_, dependencies, context)?;
        self.params.validate(context, indexes)?;
        self.validate_return_type(context, indexes)?;
        self.validate_statements(context, indexes)?;
        self.validate_name(indexes, context);
        validators::item::check_usage(ref_, &self.displayed_key(indexes), context, indexes);
        Ok(())
    }

    fn validate_name(&self, indexes: &Indexes<'_>, context: &mut ValidateContext<'_>) {
        let typeref_type = indexes.search_prelude_type("typeref");
        let may_return_typeref = match self.type_(indexes) {
            Type::Struct(struct_ref) => struct_ref == typeref_type,
            Type::Unknown => true,
            Type::NoReturn => false,
        };
        let allowed_cases: &[Case] = if may_return_typeref {
            &[Case::Snake, Case::Pascal]
        } else {
            &[Case::Snake]
        };
        validators::ident::check_case(self.name_span, allowed_cases, context);
        validators::ident::check_char_count(self.name_span, context);
    }

    fn validate_return_type(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let (Some(arrow_span), Some(return_type)) = (self.arrow_span, &self.return_type) else {
            return Ok(());
        };
        return_type.validate(Some(arrow_span), context, indexes)?;
        validators::expr::check_types(
            return_type.span(),
            return_type.type_(indexes),
            None,
            Type::Struct(indexes.search_prelude_type("typeref")),
            context,
        )?;
        Ok(())
    }

    fn validate_statements(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        for (index, statement) in self.statements.iter().enumerate() {
            statement.validate(self.const_keyword_span, context, indexes)?;
            if let Statement::Return(return_) = statement {
                validators::statement::check_return_before_end(
                    return_.span,
                    index,
                    self.statements.len(),
                    context,
                )?;
            }
        }
        if let Some(return_type) = &self.return_type {
            let return_statement = validators::statement::check_missing_return(
                &self.statements,
                self.body_end_span,
                return_type.span(),
                context,
            )?;
            validators::expr::check_types(
                return_statement.value.span(),
                return_statement.value.type_(indexes),
                Some(return_type.span()),
                self.type_(indexes),
                context,
            )?;
        } else {
            validators::statement::check_disallowed_return(
                &self.statements,
                self.name_span,
                context,
            )?;
            validators::statement::check_empty_block(&self.statements, self.body_span, context);
        }
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        let id = self.id;
        _ = write!(shader, "fn _{id}");
        self.params.transpile(shader, indexes);
        if let Some(return_type) = self
            .type_(indexes)
            .struct_ref()
            .map(StructDefinition::transpile_name)
        {
            _ = write!(shader, " -> {return_type} {{ ");
        } else {
            _ = write!(shader, " {{ ");
        }
        for statement in &self.statements {
            statement.transpile(shader, indexes);
        }
        _ = write!(shader, " }}");
    }

    pub(crate) fn transpile_name(&self, shader: &mut String) {
        let id = self.id;
        _ = write!(shader, "_{id}");
    }
}
