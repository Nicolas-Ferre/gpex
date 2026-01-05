pub(crate) mod const_;
pub(crate) mod struct_;
pub(crate) mod var;

use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::items::const_::ConstantDefinition;
use crate::language::items::struct_::StructDefinition;
use crate::language::items::var::VariableDefinition;
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemRef<'item> {
    Variable(&'item VariableDefinition),
    Constant(&'item ConstantDefinition),
    Struct(&'item StructDefinition),
}

impl NodeRef for ItemRef<'_> {
    fn file_index(&self) -> usize {
        match self {
            Self::Variable(node) => node.name_span.file_index,
            Self::Constant(node) => node.name_span.file_index,
            Self::Struct(node) => node.name_span.file_index,
        }
    }

    fn id(&self) -> u64 {
        match self {
            Self::Variable(node) => node.id,
            Self::Constant(node) => node.id,
            Self::Struct(node) => node.id,
        }
    }

    fn scope(&self) -> &[u64] {
        match self {
            Self::Variable(node) => &node.scope,
            Self::Constant(node) => &node.scope,
            Self::Struct(node) => &node.scope,
        }
    }
}

impl ItemNodeRef for ItemRef<'_> {
    fn is_public(&self) -> bool {
        match self {
            Self::Variable(node) => node.pub_keyword_span.is_some(),
            Self::Constant(node) => node.pub_keyword_span.is_some(),
            Self::Struct(node) => node.pub_keyword_span.is_some(),
        }
    }
}

impl ItemRef<'_> {
    pub(crate) fn name_span(&self) -> Span {
        match self {
            Self::Variable(node) => node.name_span,
            Self::Constant(node) => node.name_span,
            Self::Struct(node) => node.name_span,
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        match self {
            Self::Variable(node) => node.dependencies(dependencies, indexes),
            Self::Constant(node) => node.dependencies(dependencies, indexes),
            Self::Struct(_) => Ok(dependencies),
        }
    }
}
