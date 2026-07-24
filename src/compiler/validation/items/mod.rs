mod fns;

use crate::compiler::dependencies;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::{Expr, OPERATOR_FN_NAME_PREFIX};
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::validation::naming::VAR_ALLOWED_CASES;
use crate::compiler::validation::{ValidateState, exprs, logs, naming};
use crate::compiler::values::types;
use crate::compiler::values::types::Type;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{ItemNodeRef, NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::ValidateError;

// TODO: split file

pub(crate) fn validate_item<'item>(
    item: &'item Item,
    state: &mut ValidateState<'_, 'item>,
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

fn validate_params<'item>(
    params: &'item ParamGroup,
    is_compilerimpl: bool,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let mut are_params_valid = true;
    for param in &params.params {
        if validate_param(param, is_compilerimpl, state).is_err() {
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

fn validate_var<'item>(
    var: &'item VarDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Var(var);
    let mut dependencies = Dependencies::new();
    let dependency_result = dependencies::scan_var(var, &mut dependencies, state.inner);
    validate_no_circular_dependencies(ref_, dependency_result, state)?;
    validate_unique_definition(ref_, state)?;
    validate_usage(ref_, state);
    naming::validate_name(var.name_span, VAR_ALLOWED_CASES, state);
    exprs::validate_expr(&var.default_value, state)?;
    Ok(())
}

fn validate_const<'item>(
    const_: &'item ConstDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Const(const_);
    let mut dependencies = Dependencies::new();
    let dependency_result = dependencies::scan_const(const_, &mut dependencies, state.inner);
    validate_no_circular_dependencies(ref_, dependency_result, state)?;
    validate_unique_definition(ref_, state)?;
    validate_usage(ref_, state);
    let allowed_cases = naming::const_allowed_cases(const_, state);
    naming::validate_name(const_.name_span, allowed_cases, state);
    state.with_const_mark_span(Some(const_.const_keyword_span), |state| {
        exprs::validate_expr(&const_.value, state)
    })?;
    Ok(())
}

fn validate_struct<'item>(
    struct_: &'item StructDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    validate_compilerimpl_location(
        ItemRef::Struct(struct_),
        Some(struct_.compilerimpl_keyword_span),
        state,
    )?;
    Ok(())
}

fn validate_repeat(
    repeat: &RepeatDefinition,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    exprs::validate_call(&repeat.call, state)?;
    exprs::validate_has_return_type(&repeat.call, repeat.call.span, state)?;
    Ok(())
}

fn validate_param<'item>(
    param: &'item Param,
    is_compilerimpl: bool,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Param(param);
    validate_param_type(param, state)?;
    validate_param_requirement(param, state)?;
    if !is_compilerimpl {
        validate_usage(ref_, state);
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

fn validate_no_circular_dependencies(
    item: ItemRef<'_>,
    dependency_result: Result<(), Vec<Span>>,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let name_span = item.name_span();
    let name = state.context.slice(name_span);
    if let Err(stack) = dependency_result {
        if stack.iter().min() != Some(&stack[0]) {
            // avoid repeating the same error for each item of the stack
            return Err(ValidateError);
        }
        state.add_log(logs::items::circular_dependencies(
            name, name_span, &stack, state,
        ));
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn validate_compilerimpl_location(
    item: ItemRef<'_>,
    compilerimpl_keyword_span: Option<Span>,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if let Some(compilerimpl_keyword_span) = compilerimpl_keyword_span
        && item.file_index() != PRELUDE_FILE_INDEX
    {
        state.add_log(logs::items::forbidden_compilerimpl(
            compilerimpl_keyword_span,
            state,
        ));
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn validate_usage<'item>(item: ItemRef<'item>, state: &mut ValidateState<'_, 'item>) {
    let name_span = item.name_span();
    let name = state.context.slice(name_span);
    let ref_span = state.inner.item_first_refs.get(&item.id()).copied();
    let is_unused_lint_ignored =
        name.starts_with('_') && !name.starts_with(OPERATOR_FN_NAME_PREFIX);
    if !item.is_pub() && ref_span.is_none() && !is_unused_lint_ignored {
        let displayed_key = item.displayed_key(state.inner);
        state.add_log(logs::items::unused(&displayed_key, name_span, state));
    } else if item.is_pub() && is_unused_lint_ignored {
        let displayed_key = item.displayed_key(state.inner);
        state.add_log(logs::items::pub_with_ignored_name(
            &displayed_key,
            name_span,
            state,
        ));
    } else if let Some(ref_span) = ref_span
        && is_unused_lint_ignored
    {
        let displayed_key = item.displayed_key(state.inner);
        state.add_log(logs::items::used_with_ignored_name(
            &displayed_key,
            name_span,
            ref_span,
            state,
        ));
    }
}

fn validate_unique_definition<'item>(
    item: ItemRef<'item>,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let name_span = item.name_span();
    let key = item.key();
    let search_params = SearchParams {
        key: &key,
        location: item,
        imports: &state.inner.imports,
        config: SearchConfig {
            can_be_after: false,
            can_be_parent_node: false,
        },
    };
    let duplicated_item = state
        .inner
        .items
        .search_in_same_file(search_params, Visibility::Enforced)
        .next();
    if let Some(duplicated_item) = duplicated_item {
        state.add_log(logs::items::duplicate_definition(
            &key,
            name_span,
            duplicated_item.name_span(),
            state,
        ));
        Err(ValidateError)
    } else {
        Ok(())
    }
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
