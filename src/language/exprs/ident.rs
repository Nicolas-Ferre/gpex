use crate::compiler::consts::ConstValue;
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENT_PATTERN;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

const SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: false,
    can_be_parent_node: false,
};

#[derive(Debug)]
pub(crate) struct Ident {
    id: u64,
    scope: Vec<u64>,
    pub(crate) span: Span,
    slice: String,
}

impl NodeRef for &Ident {
    fn file_index(&self) -> usize {
        self.span.file_index
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn scope(&self) -> &[u64] {
        &self.scope
    }
}

impl Ident {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, IDENT_PATTERN)?;
        Ok(Self {
            id: context.next_id(),
            scope: context.scope().to_vec(),
            slice: context.slice(span).into(),
            span,
        })
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        let search_params = SearchParams {
            key: &self.slice,
            location: self,
            imports: &indexes.imports,
            config: SEARCH_CONFIG,
        };
        let matching_value = indexes
            .items
            .search(search_params, Visibility::Enforced)
            .next();
        if let Some(source) = matching_value {
            indexes.sources.insert(self.id, source);
            indexes
                .imports
                .mark_as_used(self.file_index(), source.file_index());
            indexes
                .item_first_refs
                .entry(source.id())
                .or_insert_with(|| self.span);
            if let ItemRef::Struct(struct_) = source {
                struct_.index_ref(indexes);
            }
        } else if let Some(source) = indexes
            .items
            .search(search_params, Visibility::Ignored)
            .next()
        {
            indexes.priv_sources.insert(self.id, source);
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        if let Some(&source) = indexes.sources.get(&self.id) {
            dependencies.register(self.span, source, |dependencies| {
                source.dependencies(type_, dependencies, indexes)
            })
        } else {
            Ok(dependencies)
        }
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        match indexes.sources.get(&self.id) {
            Some(source) => source.type_(indexes),
            None => Type::Unknown,
        }
    }

    pub(crate) fn const_value<'index>(&self, indexes: &Indexes<'index>) -> ConstValue<'index> {
        match indexes.sources.get(&self.id) {
            Some(ItemRef::Variable(_)) => ConstValue::RuntimeValue,
            Some(ItemRef::Constant(node)) => node.const_value(indexes),
            Some(ItemRef::Struct(node)) => ConstValue::TypeRef(node),
            Some(ItemRef::Fn(_)) => unreachable!("identifier should not refer to a function"),
            None => ConstValue::Unknown,
        }
    }

    pub(crate) fn is_ref(&self, indexes: &Indexes<'_>) -> Option<bool> {
        Some(match indexes.sources.get(&self.id)? {
            ItemRef::Variable(_) => true,
            ItemRef::Constant(_) | ItemRef::Struct(_) => false,
            ItemRef::Fn(_) => unreachable!("identifier should not refer to a function"),
        })
    }

    pub(crate) fn validate(
        &self,
        const_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::item::check_found(self, self.span, &self.slice, &self.slice, context, indexes)?;
        if let Some(const_mark_span) = const_mark_span {
            validators::expr::check_const_value(
                self.const_value(indexes),
                self.span,
                const_mark_span,
                context,
            )?;
        }
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match indexes.sources[&self.id] {
            ItemRef::Variable(node) => node.transpile_ref(shader),
            ItemRef::Fn(_) => unreachable!("identifiers cannot reference functions"),
            ItemRef::Constant(_) | ItemRef::Struct(_) => {
                unreachable!("constant item should be transpiled in `Expression::transpile`")
            }
        }
    }
}
