pub(crate) mod const_;
pub(crate) mod var;

use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::items::const_::ConstantDefinition;
use crate::language::items::var::VariableDefinition;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemRef<'item> {
    Variable(&'item VariableDefinition),
    Constant(&'item ConstantDefinition),
}

impl NodeRef for ItemRef<'_> {
    fn file_index(&self) -> usize {
        match self {
            ItemRef::Variable(node) => node.name_span.file_index,
            ItemRef::Constant(node) => node.name_span.file_index,
        }
    }

    fn id(&self) -> u64 {
        match self {
            ItemRef::Variable(node) => node.id,
            ItemRef::Constant(node) => node.id,
        }
    }

    fn scope(&self) -> &[u64] {
        match self {
            ItemRef::Variable(node) => &node.scope,
            ItemRef::Constant(node) => &node.scope,
        }
    }

    fn is_public(&self) -> bool {
        match self {
            ItemRef::Variable(node) => node.pub_keyword_span.is_some(),
            ItemRef::Constant(node) => node.pub_keyword_span.is_some(),
        }
    }
}

impl ItemRef<'_> {
    pub(crate) fn name_span(&self) -> Span {
        match self {
            ItemRef::Variable(node) => node.name_span,
            ItemRef::Constant(node) => node.name_span,
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        match self {
            ItemRef::Variable(node) => node.dependencies(dependencies, indexes),
            ItemRef::Constant(node) => node.dependencies(dependencies, indexes),
        }
    }
}
