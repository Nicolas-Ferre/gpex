use crate::compiler::consts::ConstValue;
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENT_PATTERN;
use crate::language::symbols::{CONST_KEYWORD, EQUAL_SYMBOL, PUB_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::ItemNodeRef;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use crate::validators::ident::Case;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct ConstDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) const_keyword_span: Span,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) name: String,
    #[derive_where(skip)]
    value: Expr,
}

impl ConstDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            let const_keyword_span = Span::parse_symbol(context, CONST_KEYWORD)?;
            let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
            context.force_parse_any_error();
            Span::parse_symbol(context, EQUAL_SYMBOL)?;
            let value = Expr::parse(context)?;
            Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                pub_keyword_span,
                const_keyword_span,
                name_span,
                name: context.slice(name_span).into(),
                value,
            })
        })
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(ItemRef::Constant(self));
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.value.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        self.value.dependencies(type_, dependencies, indexes)
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        self.value.type_(indexes)
    }

    pub(crate) fn const_value<'index>(&self, indexes: &Indexes<'index>) -> ConstValue<'index> {
        self.value.const_value(indexes)
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Constant(self);
        let dependencies =
            self.dependencies(DependencyType::CycleDetection, Dependencies::new(), indexes);
        validators::item::check_circular_dependencies(ref_, dependencies, context)?;
        validators::item::check_unique_definition(ref_, context, indexes)?;
        validators::item::check_usage(ref_, &ref_.key(), context, indexes);
        validators::ident::check_char_count(self.name_span, context);
        let may_return_typeref = self
            .type_(indexes)
            .struct_ref()
            .is_none_or(|type_| type_ == indexes.search_prelude_type("typeref"));
        let allowed_cases: &[Case] = if may_return_typeref {
            &[Case::ScreamingSnake, Case::Pascal]
        } else {
            &[Case::ScreamingSnake]
        };
        validators::ident::check_case(self.name_span, allowed_cases, context);
        self.value
            .validate(Some(self.const_keyword_span), context, indexes)?;
        Ok(())
    }
}
