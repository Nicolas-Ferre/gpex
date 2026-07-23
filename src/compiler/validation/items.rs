use crate::compiler::dependencies;
use crate::compiler::indexing::item_ref::ItemRef;
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
    node: &'item Item,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    debug_assert!(state.const_mark_span.is_none());
    match node {
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
    node: &'item VarDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Var(node);
    let dependency_result = dependencies::scan_var(node, state);
    validators::item::check_circular_dependencies(ref_, dependency_result, state)?;
    validators::item::check_unique_definition(ref_, state)?;
    validators::item::check_usage(ref_, state);
    validators::ident::check_char_count(node.name_span, state);
    validators::ident::check_case(node.name_span, VAR_ALLOWED_CASES, state);
    exprs::validate_expr(&node.default_value, state)?;
    Ok(())
}

fn validate_const<'item>(
    node: &'item ConstDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Const(node);
    let dependency_result = dependencies::scan_const(node, state);
    validators::item::check_circular_dependencies(ref_, dependency_result, state)?;
    validators::item::check_unique_definition(ref_, state)?;
    validators::item::check_usage(ref_, state);
    validators::ident::check_char_count(node.name_span, state);
    let allowed_cases = naming::const_allowed_cases(node, state);
    validators::ident::check_case(node.name_span, allowed_cases, state);
    state.with_const_mark_span(Some(node.const_keyword_span), |state| {
        exprs::validate_expr(&node.value, state)
    })?;
    Ok(())
}

fn validate_struct<'item>(
    node: &'item StructDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    validators::item::check_prelude_location(
        ItemRef::Struct(node),
        Some(node.compilerimpl_keyword_span),
        state,
    )?;
    Ok(())
}

fn validate_repeat(node: &RepeatDefinition, state: &mut State<'_>) -> Result<(), ValidateError> {
    exprs::validate_call(&node.call, state)?;
    validators::expr::check_has_return_type(&node.call, node.call.span, state)?;
    Ok(())
}

fn validate_param<'item>(
    node: &'item Param,
    is_compilerimpl: bool,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Param(node);
    validate_param_type(node, state)?;
    validate_param_requirement(node, state)?;
    if !is_compilerimpl {
        validators::item::check_usage(ref_, state);
    }
    validators::ident::check_char_count(node.name_span, state);
    let allowed_cases = naming::param_allowed_cases(node, state);
    validators::ident::check_case(node.name_span, allowed_cases, state);
    Ok(())
}

fn validate_param_type<'item>(
    node: &'item Param,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    if matches!(node.type_, Expr::Wildcard(_)) {
        return Ok(());
    }
    state.with_const_mark_span(Some(node.colon_span), |state| {
        exprs::validate_expr(&node.type_, state)
    })?;
    let actual_type = types::expr_type(&node.type_, state);
    let expected_type = Type::Struct(state.search_prelude_type("typeref"));
    validators::expr::check_types(node.type_.span(), actual_type, None, expected_type, state)?;
    Ok(())
}

fn validate_param_requirement<'item>(
    node: &'item Param,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let Some(requirement) = &node.requirement else {
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
