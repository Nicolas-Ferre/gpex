use crate::compiler::constants::Constant;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
pub(crate) struct Identifier {
    id: u64,
    scope: Vec<u64>,
    span: Span,
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
        if let Some(source) = indexes.items.search(&self.slice, self, imports, false) {
            indexes.sources.insert(self.id, source);
            indexes
                .item_first_refs
                .entry(source.id())
                .or_insert_with(|| self.span);
            imports.mark_as_used(self.file_index(), source.file_index());
        }
        if let Some(source) = indexes.items.search(&self.slice, self, imports, true) {
            indexes.private_sources.insert(self.id, source);
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        if let Some(&source) = indexes.sources.get(&self.id) {
            let dependencies = dependencies.register(self.span, source)?;
            source.dependencies(dependencies, indexes)
        } else {
            Ok(dependencies)
        }
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::identifier::check_found(self, self.span, context, indexes)?;
        if let Some(constant_mark_span) = constant_mark_span {
            validators::identifier::check_constant(
                self,
                self.span,
                constant_mark_span,
                context,
                indexes,
            )?;
        }
        Ok(())
    }

    pub(crate) fn constant(&self, indexes: &Indexes<'_>) -> Option<Constant> {
        match indexes.sources[&self.id] {
            ItemRef::Variable(_) => None, // no-coverage (unused for now)
            ItemRef::Constant(node) => Some(node.constant(indexes)),
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match indexes.sources[&self.id] {
            ItemRef::Variable(node) => node.transpile_ref(shader),
            ItemRef::Constant(node) => node.transpile_ref(shader, indexes),
        }
    }
}
