use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{NodeRef, SearchConfig, Visibility};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

const SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: false,
    can_be_parent_node: false,
};

#[derive(Debug)]
pub(crate) struct Identifier {
    id: u64,
    scope: Vec<u64>,
    pub(crate) span: Span,
    slice: String,
}

impl NodeRef for &Identifier {
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

impl Identifier {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
        Ok(Self {
            id: context.next_id(),
            scope: context.scope().to_vec(),
            slice: context.slice(span).into(),
            span,
        })
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        let imports = &mut indexes.imports;
        if let Some(source) = indexes.items.search(
            &self.slice,
            self,
            imports,
            Visibility::Ignored,
            SEARCH_CONFIG,
        ) {
            indexes.private_sources.insert(self.id, source);
        }
        if let Some(source) = indexes.items.search(
            &self.slice,
            self,
            imports,
            Visibility::Enforced,
            SEARCH_CONFIG,
        ) {
            imports.mark_as_used(self.file_index(), source.file_index());
            indexes.sources.insert(self.id, source);
            indexes
                .item_first_refs
                .entry(source.id())
                .or_insert_with(|| self.span);
            if let ItemRef::Struct(struct_) = source {
                struct_.index_ref(indexes);
            }
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

    pub(crate) fn type_<'index>(
        &self,
        indexes: &Indexes<'index>,
    ) -> Option<&'index StructDefinition> {
        indexes.sources.get(&self.id)?.type_(indexes)
    }

    pub(crate) fn constant<'index>(&self, indexes: &Indexes<'index>) -> Option<Constant<'index>> {
        match indexes.sources.get(&self.id)? {
            ItemRef::Variable(_) => None,
            ItemRef::Constant(node) => node.constant(indexes),
            ItemRef::Struct(node) => Some(Constant::TypeRef(node)),
            ItemRef::Function(_) => unreachable!("identifier should not refer to a function"),
        }
    }

    pub(crate) fn is_ref(&self, indexes: &Indexes<'_>) -> Option<bool> {
        Some(match indexes.sources.get(&self.id)? {
            ItemRef::Variable(_) => true,
            ItemRef::Constant(_) | ItemRef::Struct(_) => false,
            ItemRef::Function(_) => unreachable!("identifier should not refer to a function"),
        })
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::item::check_found(self, self.span, &self.slice, context, indexes)?;
        if let Some(constant_mark_span) = constant_mark_span {
            validators::expression::check_constant(
                self.constant(indexes),
                self.span,
                constant_mark_span,
                context,
            )?;
        }
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match indexes.sources[&self.id] {
            ItemRef::Variable(node) => node.transpile_ref(shader),
            ItemRef::Function(_) => unreachable!("identifiers cannot reference functions"),
            ItemRef::Constant(_) | ItemRef::Struct(_) => {
                unreachable!("constant item should be transpiled in `Expression::transpile`")
            }
        }
    }
}
