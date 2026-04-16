use crate::compiler::consts::ConstContext;
use crate::compiler::indexes::Indexes;
use crate::compiler::transpilation::MAIN_BUFFER_NAME;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENT_PATTERN;
use crate::language::symbols::{EQUAL_SYMBOL, PUB_KEYWORD, SEMICOLON_SYMBOL, VAR_KEYWORD};
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::ItemNodeRef;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use crate::validators::ident::Case;
use std::fmt::Write;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VarDefinition {
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
    default_value: Expr,
}

impl VarDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let scope = context.scope().to_vec();
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            Span::parse_symbol(context, VAR_KEYWORD)?;
            context.force_parse_any_error();
            let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
            Span::parse_symbol(context, EQUAL_SYMBOL)?;
            let default_value = Expr::parse(context)?;
            Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
            Ok(Self {
                id,
                scope,
                pub_keyword_span,
                name_span,
                name: context.slice(name_span).into(),
                default_value,
            })
        })
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(ItemRef::Variable(self));
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        self.default_value.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        self.default_value
            .dependencies(type_, dependencies, indexes)
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        self.default_value.type_(indexes)
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Variable(self);
        let dependencies =
            self.dependencies(DependencyType::CycleDetection, Dependencies::new(), indexes);
        validators::item::check_circular_dependencies(ref_, dependencies, context)?;
        validators::item::check_unique_definition(ref_, context, indexes)?;
        validators::item::check_usage(ref_, &ref_.key(), context, indexes);
        validators::ident::check_char_count(self.name_span, context);
        validators::ident::check_case(self.name_span, &[Case::Snake], context);
        self.default_value.validate(None, context, indexes)?;
        Ok(())
    }

    pub(crate) fn transpile_buffer_field(&self, shader: &mut String, indexes: &Indexes<'_>) {
        let type_ = self
            .type_(indexes)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("variable type should be validated before"));
        _ = write!(shader, "v{}: {}, ", self.id, type_.transpile_name());
    }

    pub(crate) fn transpile_buffer_init(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.transpile_ref(shader);
        *shader += " = ";
        self.default_value
            .transpile(shader, indexes, &mut ConstContext::default());
        *shader += "; ";
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String) {
        *shader += MAIN_BUFFER_NAME;
        _ = write!(shader, ".v{}", self.id);
    }
}
