use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::symbols::QUESTION_MARK_SYMBOL;
use crate::compiler::types::TypeResolver;
use crate::utils::validation::ValidateError;

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

    pub(crate) fn call_key(&mut self, node: &Call) -> Result<String, ValidateError> {
        let fn_name = &node.name;
        let arg_types = node
            .args
            .iter()
            .map(|arg| self.type_resolver.expr_type(arg).name())
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        Ok(format!("{fn_name}({arg_types})"))
    }

    pub(crate) fn fn_key(&mut self, node: &FnDefinition) -> Result<String, ValidateError> {
        let fn_name = &node.name;
        let param_types = node
            .params
            .params
            .iter()
            .map(|param| {
                if matches!(param.type_, Expr::Wildcard(_)) {
                    Ok(QUESTION_MARK_SYMBOL.slice.into())
                } else {
                    self.type_resolver.expr_as_type(&param.type_).name()
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        Ok(format!("{fn_name}({param_types})"))
    }
}
