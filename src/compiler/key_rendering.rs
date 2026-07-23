use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::symbols::QUESTION_MARK_SYMBOL;
use crate::compiler::state::State;
use crate::compiler::values::types::{expr_as_type, expr_type};
use crate::utils::validation::ValidateError;

pub(crate) fn call_key(call: &Call, state: &mut State<'_>) -> Result<String, ValidateError> {
    let fn_name = &call.name;
    let arg_types = call
        .args
        .iter()
        .map(|arg| expr_type(&arg.value, state).name())
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("{fn_name}({arg_types})"))
}

pub(crate) fn fn_key(fn_: &FnDefinition, state: &mut State<'_>) -> Result<String, ValidateError> {
    let fn_name = &fn_.name;
    let param_types = fn_
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
