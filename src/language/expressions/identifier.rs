use crate::compiler::constants::Constant;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

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
        if let Some(source) = indexes.items.search(&self.slice, self, imports, true) {
            indexes.private_sources.insert(self.id, source);
        }
        if let Some(source) = indexes.items.search(&self.slice, self, imports, false) {
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

    pub(crate) fn type_<'index>(
        &self,
        indexes: &Indexes<'index>,
    ) -> Option<&'index StructDefinition> {
        match indexes.sources.get(&self.id)? {
            ItemRef::Variable(node) => node.type_(indexes),
            ItemRef::Constant(node) => node.type_(indexes),
            ItemRef::Struct(_) => Some(StructDefinition::type_(indexes)),
            ItemRef::Function(node) => node.type_(indexes),
        }
    }

    pub(crate) fn constant<'index>(&self, indexes: &Indexes<'index>) -> Option<Constant<'index>> {
        match indexes.sources[&self.id] {
            ItemRef::Variable(_) | ItemRef::Function(_) => None, // no-coverage (unused for now)
            ItemRef::Constant(node) => node.constant(indexes),
            ItemRef::Struct(node) => Some(Constant::TypeRef(node)),
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

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match indexes.sources[&self.id] {
            ItemRef::Variable(node) => node.transpile_ref(shader),
            ItemRef::Constant(node) => node.transpile_ref(shader, indexes),
            ItemRef::Struct(node) => node.transpile_ref(shader),
            ItemRef::Function(_) => unreachable!("functions cannot yet be called"),
        }
    }
}
