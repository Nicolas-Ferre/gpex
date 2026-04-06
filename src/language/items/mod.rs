pub(crate) mod const_;
pub(crate) mod fn_;
pub(crate) mod param;
pub(crate) mod repeat;
pub(crate) mod struct_;
pub(crate) mod var;

use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::const_::ConstDefinition;
use crate::language::items::fn_::FnDefinition;
use crate::language::items::struct_::StructDefinition;
use crate::language::items::var::VarDefinition;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemRef<'item> {
    Variable(&'item VarDefinition),
    Constant(&'item ConstDefinition),
    Struct(&'item StructDefinition),
    Fn(&'item FnDefinition),
}

impl NodeRef for ItemRef<'_> {
    fn file_index(&self) -> usize {
        match self {
            Self::Variable(node) => node.name_span.file_index,
            Self::Constant(node) => node.name_span.file_index,
            Self::Struct(node) => node.name_span.file_index,
            Self::Fn(node) => node.name_span.file_index,
        }
    }

    fn id(&self) -> u64 {
        match self {
            Self::Variable(node) => node.id,
            Self::Constant(node) => node.id,
            Self::Struct(node) => node.id,
            Self::Fn(node) => node.id,
        }
    }

    fn scope(&self) -> &[u64] {
        match self {
            Self::Variable(node) => &node.scope,
            Self::Constant(node) => &node.scope,
            Self::Struct(node) => &node.scope,
            Self::Fn(_) => unreachable!("function calls should ignore source scope"),
        }
    }
}

impl ItemNodeRef for ItemRef<'_> {
    fn is_pub(&self) -> bool {
        match self {
            Self::Variable(node) => node.pub_keyword_span.is_some(),
            Self::Constant(node) => node.pub_keyword_span.is_some(),
            Self::Struct(node) => node.pub_keyword_span.is_some(),
            Self::Fn(node) => node.pub_keyword_span.is_some(),
        }
    }

    fn key(&self) -> String {
        match self {
            ItemRef::Variable(node) => node.name.clone(),
            ItemRef::Constant(node) => node.name.clone(),
            ItemRef::Struct(node) => node.name.clone(),
            ItemRef::Fn(node) => node.key(),
        }
    }
}

impl ItemRef<'_> {
    pub(crate) fn name_span(self) -> Span {
        match self {
            Self::Variable(node) => node.name_span,
            Self::Constant(node) => node.name_span,
            Self::Struct(node) => node.name_span,
            Self::Fn(node) => node.name_span,
        }
    }

    pub(crate) fn has_same_param_types_as(&self, args: &[Expr], indexes: &Indexes<'_>) -> bool {
        let params = match self {
            ItemRef::Fn(node) => &node.params,
            ItemRef::Variable(_) | ItemRef::Constant(_) | ItemRef::Struct(_) => {
                unreachable!("only functions can have parameters")
            }
        };
        debug_assert_eq!(params.params.len(), args.len());
        params
            .params
            .iter()
            .zip(args)
            .all(|(param, arg)| param.type_(indexes) == arg.type_(indexes))
    }

    pub(crate) fn dependencies<'index>(
        self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        match self {
            Self::Variable(node) => node.dependencies(type_, dependencies, indexes),
            Self::Constant(node) => node.dependencies(type_, dependencies, indexes),
            Self::Struct(_) => Ok(dependencies),
            Self::Fn(node) => node.dependencies(type_, dependencies, indexes),
        }
    }

    pub(crate) fn type_<'index>(self, indexes: &Indexes<'index>) -> Type<'index> {
        match self {
            ItemRef::Variable(node) => node.type_(indexes),
            ItemRef::Constant(node) => node.type_(indexes),
            ItemRef::Struct(_) => Type::Struct(StructDefinition::type_(indexes)),
            ItemRef::Fn(node) => node.type_(indexes),
        }
    }

    pub(crate) fn transpile(self, shader: &mut String, indexes: &Indexes<'_>) {
        match self {
            Self::Fn(node) => node.transpile(shader, indexes),
            Self::Variable(_) | Self::Constant(_) | Self::Struct(_) => (),
        }
    }
}
