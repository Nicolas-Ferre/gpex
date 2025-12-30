use crate::compiler::constants::Constant;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{CONST_KEYWORD, EQUAL_SYMBOL, SEMICOLON_SYMBOL};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct ConstantDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) const_keyword: Span,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    name: String,
    #[derive_where(skip)]
    value: Expression,
}

impl ConstantDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let const_keyword = Span::parse_symbol(context, CONST_KEYWORD)?;
            let name_span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
            Span::parse_symbol(context, EQUAL_SYMBOL)?;
            let value = Expression::parse(context)?;
            Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                const_keyword,
                name: context.slice(name_span).into(),
                name_span,
                value,
            })
        })
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(&self.name, ItemRef::Constant(self));
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.value.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        self.value.dependencies(dependencies, indexes)
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Constant(self);
        let dependencies = self.dependencies(Dependencies::new(ref_), indexes);
        validators::item::check_circular_dependencies(ref_, dependencies, context)?;
        validators::item::check_unique_definition(ref_, context, indexes)?;
        validators::item::check_usage(ref_, context, indexes);
        validators::identifier::check_char_count(self.name_span, context);
        validators::identifier::check_screaming_snake_case(self.name_span, context);
        self.value
            .validate(Some(self.const_keyword), context, indexes)?;
        Ok(())
    }

    #[expect(clippy::expect_used)] // validated before
    pub(crate) fn constant(&self, indexes: &Indexes<'_>) -> Constant {
        self.value
            .constant(indexes)
            .expect("internal error: invalid constant value")
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.constant(indexes).transpile(shader);
    }
}
