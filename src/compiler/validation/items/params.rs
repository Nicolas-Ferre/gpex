use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::types;
use crate::compiler::types::Type;
use crate::compiler::validation::{ValidateState, exprs, items, logs, naming};
use crate::utils::validation::ValidateError;

pub(super) fn validate_params<'item>(
    params: &'item ParamGroup,
    is_intrinsic: bool,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let mut are_params_valid = true;
    for param in &params.params {
        if validate_param(param, is_intrinsic, state).is_err() {
            are_params_valid = false;
        }
    }
    validate_unique_params(&params.params, state)?;
    if are_params_valid {
        Ok(())
    } else {
        Err(ValidateError)
    }
}

fn validate_param<'item>(
    param: &'item Param,
    is_intrinsic: bool,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Param(param);
    validate_param_type(param, state)?;
    validate_param_requirement(param, state)?;
    if !is_intrinsic {
        items::validate_usage(ref_, state);
    }
    let allowed_cases = naming::param_allowed_cases(param, state);
    naming::validate_name(param.name_span, allowed_cases, state);
    Ok(())
}

fn validate_param_type<'item>(
    param: &'item Param,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    if matches!(param.type_, Expr::Wildcard(_)) {
        return Ok(());
    }
    state.with_const_mark_span(Some(param.colon_span), |state| {
        exprs::validate_expr(&param.type_, state)
    })?;
    let actual_type = types::expr_type(&param.type_, state.inner);
    let expected_type = Type::Struct(state.inner.search_prelude_type("typeref"));
    exprs::validate_type_match(param.type_.span(), actual_type, None, expected_type, state)?;
    Ok(())
}

fn validate_param_requirement<'item>(
    param: &'item Param,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let Some(requirement) = &param.requirement else {
        return Ok(());
    };
    state.with_const_mark_span(Some(requirement.require_span), |state| {
        exprs::validate_expr(&requirement.condition, state)
    })?;
    let actual_type = types::expr_type(&requirement.condition, state.inner);
    let expected_type = Type::Struct(state.inner.search_prelude_type("bool"));
    exprs::validate_type_match(
        requirement.condition.span(),
        actual_type,
        None,
        expected_type,
        state,
    )?;
    Ok(())
}

fn validate_unique_params(
    params: &[Param],
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let mut is_error = false;
    for (param_index, param) in params.iter().enumerate() {
        let duplicated_param = params[..param_index]
            .iter()
            .find(|other_param| other_param.name == param.name);
        if let Some(duplicated_param) = duplicated_param {
            state.add_log(logs::items::duplicate_param(
                &param.name,
                param.name_span,
                duplicated_param.name_span,
                state,
            ));
            is_error = true;
        }
    }
    if is_error { Err(ValidateError) } else { Ok(()) }
}
