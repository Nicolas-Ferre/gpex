use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::types::{Type, TypeResolver};
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

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

    pub(crate) fn call_signature_span(self) -> Span {
        match self {
            Self::Fn(node) => node.signature_span,
            // coverage: off (only functions can be called)
            Self::Var(_) | Self::Const(_) | Self::Struct(_) | Self::Param(_) => {
                unreachable!("only functions can be called")
            } // coverage: on
        }
    }

    pub(crate) fn has_same_param_types_as_args(self, args: &[Expr], indexes: &Indexes<'_>) -> bool {
        let params = self.params();
        debug_assert_eq!(params.params.len(), args.len());
        let mut type_resolver = TypeResolver::new(indexes);
        type_resolver.const_resolver.enter_scope();
        for (param, arg) in params.params.iter().zip(args) {
            let param_type = type_resolver.param_type(param);
            let arg_type = type_resolver.expr_type(arg);
            if !matches!(param_type, Type::Wildcard(_)) && param_type != arg_type {
                return false;
            }
            if param.const_mark_span().is_some() {
                let value = type_resolver.const_resolver.expr_value(arg);
                type_resolver.const_resolver.add_value(param.id, value);
            }
        }
        true
    }

    pub(crate) fn params(self) -> &'item ParamGroup {
        match self {
            ItemRef::Fn(item) => &item.params,
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("only functions can have parameters")
            }
        }
    }

    pub(crate) fn displayed_key(
        self,
        key_renderer: &mut KeyRenderer<'_, '_>,
    ) -> Result<String, ValidateError> {
        match self {
            ItemRef::Fn(item) => key_renderer.fn_key(item),
            ItemRef::Var(item) => Ok(item.name.clone()),
            ItemRef::Const(item) => Ok(item.name.clone()),
            ItemRef::Param(item) => Ok(item.name.clone()),
            ItemRef::Struct(item) => Ok(item.name.clone()),
        }
    }
}
