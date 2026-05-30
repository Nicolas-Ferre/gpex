use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::types::TypeResolver;
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ItemRef<'item> {
    Var(&'item VarDefinition),
    Const(&'item ConstDefinition),
    Struct(&'item StructDefinition),
    Fn(&'item FnDefinition),
    Param(&'item Param),
}

impl NodeRef for ItemRef<'_> {
    fn file_index(&self) -> usize {
        match self {
            Self::Var(node) => node.name_span.file_index,
            Self::Const(node) => node.name_span.file_index,
            Self::Struct(node) => node.name_span.file_index,
            Self::Fn(node) => node.name_span.file_index,
            Self::Param(node) => node.name_span.file_index,
        }
    }

    fn id(&self) -> u64 {
        match self {
            Self::Var(node) => node.id,
            Self::Const(node) => node.id,
            Self::Struct(node) => node.id,
            Self::Fn(node) => node.id,
            Self::Param(node) => node.id,
        }
    }

    fn scope(&self) -> &[u64] {
        match self {
            Self::Var(node) => &node.scope,
            Self::Const(node) => &node.scope,
            Self::Struct(node) => &node.scope,
            Self::Fn(node) => &node.scope,
            Self::Param(node) => &node.scope,
        }
    }
}

impl ItemNodeRef for ItemRef<'_> {
    fn is_pub(&self) -> bool {
        match self {
            Self::Var(node) => node.pub_keyword_span.is_some(),
            Self::Const(node) => node.pub_keyword_span.is_some(),
            Self::Struct(node) => node.pub_keyword_span.is_some(),
            Self::Fn(node) => node.pub_keyword_span.is_some(),
            Self::Param(_) => false,
        }
    }

    fn key(&self) -> String {
        match self {
            Self::Var(node) => node.name.clone(),
            Self::Const(node) => node.name.clone(),
            Self::Struct(node) => node.name.clone(),
            Self::Fn(node) => node.key(),
            Self::Param(node) => node.name.clone(),
        }
    }
}

impl<'item> ItemRef<'item> {
    pub(crate) fn name_span(self) -> Span {
        match self {
            Self::Var(node) => node.name_span,
            Self::Const(node) => node.name_span,
            Self::Struct(_) => unreachable!("struct name span is never used"),
            Self::Fn(node) => node.name_span,
            Self::Param(node) => node.name_span,
        }
    }

    pub(crate) fn has_same_param_types_as_args(self, args: &[Expr], indexes: &Indexes<'_>) -> bool {
        let params = self.params();
        debug_assert_eq!(params.params.len(), args.len());
        let type_resolver = TypeResolver::new(indexes);
        params
            .params
            .iter()
            .zip(args)
            .all(|(param, arg)| type_resolver.param_type(param) == type_resolver.expr_type(arg))
    }

    pub(crate) fn params(self) -> &'item ParamGroup {
        match self {
            ItemRef::Fn(item) => &item.params,
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("only functions can have parameters")
            }
        }
    }
}
