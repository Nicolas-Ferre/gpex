use crate::compiler::transversal::ast::exprs::Expr;
use crate::compiler::transversal::ast::exprs::calls::Call;
use crate::compiler::transversal::ast::items::ItemRef;
use crate::compiler::transversal::ast::items::fns::FnDefinition;
use crate::compiler::transversal::ast::symbols::QUESTION_MARK_SYMBOL;
use crate::compiler::transversal::state::State;
use crate::compiler::transversal::types;
use crate::utils::validation::ValidateError;

pub(crate) fn item_key<'item>(item: ItemRef<'item>, state: &State<'item>) -> String {
    match item {
        ItemRef::Fn(fn_) => fn_key(fn_, state)
            .unwrap_or_else(|_| unreachable!("function should be validated before")),
        ItemRef::Var(var) => var.name.clone(),
        ItemRef::Const(const_) => const_.name.clone(),
        ItemRef::Param(param) => param.name.clone(),
        ItemRef::Struct(_) => unreachable!("structs are not yet validated"),
    }
}

pub(crate) fn call_key(call: &Call, state: &State<'_>) -> Result<String, ValidateError> {
    let fn_name = &call.name;
    let arg_types = call
        .args
        .iter()
        .map(|arg| types::expr_type(&arg.value, state).name())
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("{fn_name}({arg_types})"))
}

pub(crate) fn fn_key(fn_: &FnDefinition, state: &State<'_>) -> Result<String, ValidateError> {
    let fn_name = &fn_.name;
    let param_types = fn_
        .params
        .params
        .iter()
        .map(|param| {
            if matches!(param.type_, Expr::Wildcard(_)) {
                Ok(QUESTION_MARK_SYMBOL.slice.into())
            } else {
                types::expr_as_type(&param.type_, state).name()
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("{fn_name}({param_types})"))
}
