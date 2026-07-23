use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::symbols::QUESTION_MARK_SYMBOL;
use crate::compiler::state::State;
use crate::compiler::values::types::{expr_as_type, expr_type};
use crate::utils::validation::ValidateError;

pub(crate) fn call_key(node: &Call, state: &mut State<'_>) -> Result<String, ValidateError> {
    let fn_name = &node.name;
    let arg_types = node
        .args
        .iter()
        .map(|arg| expr_type(&arg.value, state).name())
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("{fn_name}({arg_types})"))
}

pub(crate) fn fn_key(node: &FnDefinition, state: &mut State<'_>) -> Result<String, ValidateError> {
    let fn_name = &node.name;
    let param_types = node
        .params
        .params
        .iter()
        .map(|param| {
            if matches!(param.type_, Expr::Wildcard(_)) {
                Ok(QUESTION_MARK_SYMBOL.slice.into())
            } else {
                expr_as_type(&param.type_, state).name()
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("{fn_name}({param_types})"))
}
