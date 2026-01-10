use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::statements::return_::ReturnStatement;
use crate::language::symbols::{
    ARROW_SYMBOL, BRACE_CLOSE_SYMBOL, BRACE_OPEN_SYMBOL, FN_KEYWORD, PARENTHESIS_CLOSE_SYMBOL,
    PARENTHESIS_OPEN_SYMBOL, PUB_KEYWORD,
};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct FunctionDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    name: String,
    #[derive_where(skip)]
    arrow_span: Span,
    #[derive_where(skip)]
    return_type: Expression,
    #[derive_where(skip)]
    statements: Vec<ReturnStatement>,
    #[derive_where(skip)]
    body_end_span: Span,
}

impl FunctionDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            Span::parse_symbol(context, FN_KEYWORD)?;
            let name_span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
            Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
            Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
            let arrow_span = Span::parse_symbol(context, ARROW_SYMBOL)?;
            let return_type = Expression::parse(context)?;
            Span::parse_symbol(context, BRACE_OPEN_SYMBOL)?;
            let (statements, _) = context.parse_many(0, ReturnStatement::parse, None)?;
            let body_end_span = Span::parse_symbol(context, BRACE_CLOSE_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                pub_keyword_span,
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

    pub(crate) fn type_<'index>(
        &self,
        indexes: &Indexes<'index>,
    ) -> Option<&'index StructDefinition> {
        self.return_type.constant(indexes)?.type_ref()
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Function(self);
        validators::item::check_unique_definition(ref_, context, indexes)?;
        self.validate_return_type(context, indexes)?;
        self.validate_statements(context, indexes)?;
        Ok(())
    }

    fn validate_return_type(
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
        Ok(())
    }

    fn validate_statements(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        for (index, statement) in self.statements.iter().enumerate() {
            statement.validate(context, indexes)?;
            validators::statements::check_return(
                statement.span,
                index,
                self.statements.len(),
                context,
            )?;
        }
        validators::statements::check_missing_return(
            self.statements.is_empty(),
            self.body_end_span,
            self.return_type.span(),
            context,
        )?;
        let last_statement = &self.statements[self.statements.len() - 1];
        let returned_type = last_statement.value.type_(indexes);
        let expected_type = self.type_(indexes);
        validators::expression::check_types(
            last_statement.value.span(),
            returned_type,
            Some(self.return_type.span()),
            expected_type,
            context,
        )?;
        Ok(())
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        let mut dependencies = self.return_type.dependencies(dependencies, indexes)?;
        for statement in &self.statements {
            dependencies = statement.dependencies(dependencies, indexes)?;
        }
        Ok(dependencies)
    }
}
