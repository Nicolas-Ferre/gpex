use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::compiler::transpilation::MAIN_BUFFER_NAME;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{EQUAL_SYMBOL, PUB_KEYWORD, SEMICOLON_SYMBOL, VAR_KEYWORD};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use std::fmt::Write;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VariableDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) name: String,
    #[derive_where(skip)]
    default_value: Expression,
}

impl VariableDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            Span::parse_symbol(context, VAR_KEYWORD)?;
            let name_span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
            Span::parse_symbol(context, EQUAL_SYMBOL)?;
            let default_value = Expression::parse(context)?;
            Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                name: context.slice(name_span).into(),
                pub_keyword_span,
                name_span,
                default_value,
            })
        })
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(&self.name, ItemRef::Variable(self));
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.default_value.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        self.default_value.dependencies(dependencies, indexes)
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Variable(self);
        let dependencies = self.dependencies(Dependencies::new(ref_), indexes);
        validators::item::check_circular_dependencies(ref_, dependencies, context)?;
        validators::item::check_unique_definition(ref_, context, indexes)?;
        validators::item::check_usage(ref_, context, indexes);
        validators::identifier::check_char_count(self.name_span, context);
        validators::identifier::check_snake_case(self.name_span, context);
        self.default_value.validate(None, context, indexes)?;
        Ok(())
    }

    pub(crate) fn transpile_buffer_field(&self, shader: &mut String) {
        _ = write!(shader, "v{}: i32, ", self.id);
    }

    pub(crate) fn transpile_buffer_init(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.transpile_ref(shader);
        *shader += " = ";
        self.default_value.transpile(shader, indexes);
        *shader += "; ";
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String) {
        *shader += MAIN_BUFFER_NAME;
        _ = write!(shader, ".v{}", self.id);
    }
}
