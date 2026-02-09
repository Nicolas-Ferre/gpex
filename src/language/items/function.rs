use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::statements::Statement;
use crate::language::statements::return_::ReturnStatement;
use crate::language::symbols::{
    ARROW_SYMBOL, BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, CONST_KEYWORD, FN_KEYWORD,
    PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL, PUB_KEYWORD,
};
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use crate::validators::identifier::Case;
use std::fmt::Write;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct FunctionDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) const_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    name: String,
    #[derive_where(skip)]
    arrow_span: Span,
    #[derive_where(skip)]
    return_type: Expression,
    #[derive_where(skip)]
    statements: Vec<Statement>,
    #[derive_where(skip)]
    body_end_span: Span,
}

impl FunctionDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            let const_keyword_span = Span::parse_symbol(context, CONST_KEYWORD).ok();
            Span::parse_symbol(context, FN_KEYWORD)?;
            let name_span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
            Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
            Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
            let arrow_span = Span::parse_symbol(context, ARROW_SYMBOL)?;
            let return_type = Expression::parse(context)?;
            Span::parse_symbol(context, BRACE_OPEN_SYMBOL)?;
            let (statements, _) = context.parse_many(0, Statement::parse, None)?;
            let body_end_span = Span::parse_symbol(context, BRACE_CLOSE_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                pub_keyword_span,
                const_keyword_span,
                name_span,
                name: context.slice(name_span).into(),
                arrow_span,
                return_type,
                statements,
                body_end_span,
            })
        })
    }

    pub(crate) fn key(&self) -> String {
        format!("{}()", self.name)
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(ItemRef::Function(self));
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.return_type.index(indexes);
        for statement in &self.statements {
            statement.index(indexes);
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        let mut dependencies = self
            .return_type
            .dependencies(type_, dependencies, indexes)?;
        for statement in &self.statements {
            dependencies = statement.dependencies(type_, dependencies, indexes)?;
        }
        Ok(dependencies)
    }

    pub(crate) fn type_<'index>(
        &self,
        indexes: &Indexes<'index>,
    ) -> Option<&'index StructDefinition> {
        self.return_type.constant(indexes)?.type_ref()
    }

    pub(crate) fn constant<'index>(&self, indexes: &Indexes<'index>) -> Option<Constant<'index>> {
        self.const_keyword_span.and_then(|_| {
            self.return_statement()
                .and_then(|return_| return_.value.constant(indexes))
        })
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Function(self);
        let dependencies =
            self.dependencies(DependencyType::CycleDetection, Dependencies::new(), indexes);
        validators::item::check_circular_dependencies(ref_, dependencies, context)?;
        validators::item::check_unique_definition(ref_, context, indexes)?;
        validators::item::check_usage(ref_, context, indexes);
        let signature_result = self.validate_signature(context, indexes);
        let statements_result = self.validate_statements(context, indexes);
        signature_result.and(statements_result)
    }

    fn validate_signature(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let typeref_type = indexes.search_prelude_type("typeref");
        self.return_type
            .validate(Some(self.arrow_span), context, indexes)?;
        validators::expression::check_types(
            self.return_type.span(),
            self.return_type.type_(indexes),
            None,
            Some(typeref_type),
            context,
        )?;
        validators::identifier::check_char_count(self.name_span, context);
        let may_return_typeref = self
            .type_(indexes)
            .is_none_or(|type_| type_ == typeref_type);
        let allowed_cases: &[Case] = if may_return_typeref {
            &[Case::Snake, Case::Pascal]
        } else {
            &[Case::Snake]
        };
        validators::identifier::check_case(self.name_span, allowed_cases, context);
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
        let return_statement = validators::statement::check_missing_return(
            &self.statements,
            self.body_end_span,
            self.return_type.span(),
            context,
        )?;
        validators::expression::check_types(
            return_statement.value.span(),
            return_statement.value.type_(indexes),
            Some(self.return_type.span()),
            self.type_(indexes),
            context,
        )?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        let id = self.id;
        let return_type = self
            .type_(indexes)
            .unwrap_or_else(|| unreachable!("return type validated before"))
            .transpile_name();
        _ = write!(shader, "fn _{id}() -> {return_type} {{ ");
        for statement in &self.statements {
            statement.transpile(shader, indexes);
        }
        _ = write!(shader, " }}");
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String) {
        _ = write!(shader, "_{}()", self.id);
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
}
