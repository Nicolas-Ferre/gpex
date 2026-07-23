use crate::compiler::dependencies;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::state::State;
use crate::compiler::validation::naming::VAR_ALLOWED_CASES;
use crate::compiler::validation::{exprs, fns, naming, validators};
use crate::compiler::values::types;
use crate::compiler::values::types::Type;
use crate::utils::validation::ValidateError;

pub(crate) fn validate_item<'item>(
    item: &'item Item,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    debug_assert!(state.const_mark_span.is_none());
    match item {
        Item::Import(_) => Ok(()), // validated during previous pass
        Item::Var(item) => validate_var(item, state),
        Item::Const(item) => validate_const(item, state),
        Item::Struct(item) => validate_struct(item, state),
        Item::Fn(item) => fns::validate_fn(item, state),
        Item::Repeat(item) => validate_repeat(item, state),
    }
}

pub(crate) fn validate_params<'item>(
    params: &'item ParamGroup,
    is_compilerimpl: bool,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let mut are_params_valid = true;
    for param in &params.params {
        if validate_param(param, is_compilerimpl, state).is_err() {
            are_params_valid = false;
        }
    }
    validators::item::check_unique_params(&params.params, state)?;
    if are_params_valid {
        Ok(())
    } else {
        Err(ValidateError)
    }
}

fn validate_var<'item>(
    var: &'item VarDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Var(var);
    let dependency_result = dependencies::scan_var(var, state);
    validators::item::check_circular_dependencies(ref_, dependency_result, state)?;
    validators::item::check_unique_definition(ref_, state)?;
    validators::item::check_usage(ref_, state);
    validators::ident::check_char_count(var.name_span, state);
    validators::ident::check_case(var.name_span, VAR_ALLOWED_CASES, state);
    exprs::validate_expr(&var.default_value, state)?;
    Ok(())
}

fn validate_const<'item>(
    const_: &'item ConstDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Const(const_);
    let dependency_result = dependencies::scan_const(const_, state);
    validators::item::check_circular_dependencies(ref_, dependency_result, state)?;
    validators::item::check_unique_definition(ref_, state)?;
    validators::item::check_usage(ref_, state);
    validators::ident::check_char_count(const_.name_span, state);
    let allowed_cases = naming::const_allowed_cases(const_, state);
    validators::ident::check_case(const_.name_span, allowed_cases, state);
    state.with_const_mark_span(Some(const_.const_keyword_span), |state| {
        exprs::validate_expr(&const_.value, state)
    })?;
    Ok(())
}

fn validate_struct<'item>(
    struct_: &'item StructDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    validators::item::check_prelude_location(
        ItemRef::Struct(struct_),
        Some(struct_.compilerimpl_keyword_span),
        state,
    )?;
    Ok(())
}

fn validate_repeat(repeat: &RepeatDefinition, state: &mut State<'_>) -> Result<(), ValidateError> {
    exprs::validate_call(&repeat.call, state)?;
    validators::expr::check_has_return_type(&repeat.call, repeat.call.span, state)?;
    Ok(())
}

fn validate_param<'item>(
    param: &'item Param,
    is_compilerimpl: bool,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Param(param);
    validate_param_type(param, state)?;
    validate_param_requirement(param, state)?;
    if !is_compilerimpl {
        validators::item::check_usage(ref_, state);
    }
    validators::ident::check_char_count(param.name_span, state);
    let allowed_cases = naming::param_allowed_cases(param, state);
    validators::ident::check_case(param.name_span, allowed_cases, state);
    Ok(())
}

fn validate_param_type<'item>(
    param: &'item Param,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    if matches!(param.type_, Expr::Wildcard(_)) {
        return Ok(());
    }
    state.with_const_mark_span(Some(param.colon_span), |state| {
        exprs::validate_expr(&param.type_, state)
    })?;
    let actual_type = types::expr_type(&param.type_, state);
    let expected_type = Type::Struct(state.search_prelude_type("typeref"));
    validators::expr::check_types(param.type_.span(), actual_type, None, expected_type, state)?;
    Ok(())
}

fn validate_param_requirement<'item>(
    param: &'item Param,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let Some(requirement) = &param.requirement else {
        return Ok(());
    };
    state.with_const_mark_span(Some(requirement.require_span), |state| {
        exprs::validate_expr(&requirement.condition, state)
    })?;
    let actual_type = types::expr_type(&requirement.condition, state);
    let expected_type = Type::Struct(state.search_prelude_type("bool"));
    validators::expr::check_types(
        requirement.condition.span(),
        actual_type,
        None,
        expected_type,
        state,
    )?;
    Ok(())
}
