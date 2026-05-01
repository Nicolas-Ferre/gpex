use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::types::TypeResolver;
use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct KeyRenderer<'item, 'index> {
    type_resolver: TypeResolver<'item, 'index>,
}

impl<'item, 'index> KeyRenderer<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            type_resolver: TypeResolver::new(indexes),
        }
    }

    pub(crate) fn call_key(&self, node: &Call) -> String {
        let fn_name = &node.name;
        let arg_types = node
            .args
            .iter()
            .map(|arg| self.type_resolver.expr_type(arg).name())
            .join(", ");
        format!("{fn_name}({arg_types})")
    }

    pub(crate) fn fn_key(&self, node: &FnDefinition) -> String {
        let fn_name = &node.name;
        let param_types = node
            .params
            .params
            .iter()
            .map(|param| self.type_resolver.expr_as_type(&param.type_).name())
            .join(", ");
        format!("{fn_name}({param_types})")
    }
}
