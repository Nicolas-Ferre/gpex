use crate::compiler::dependencies;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::BINARY_FN_NAMES;
use crate::compiler::parsing::exprs::calls::UNARY_FN_NAMES;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::queries;
use crate::compiler::types::{self, Type};
use crate::compiler::validation::items::{params, statements};
use crate::compiler::validation::{ParamConstness, ValidateState, exprs, items, logs, naming};
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{SearchConfig, SearchParams, Visibility};
use crate::utils::validation::ValidateError;

pub(super) fn validate_fn<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Fn(fn_);
    let intrinsic_span = fn_.body.intrinsic_keyword_span();
    let mut dependencies = Dependencies::new();
    let dependency_result = dependencies::scan_fn(fn_, &mut dependencies, state.inner);
    items::validate_no_circular_dependencies(ref_, dependency_result, state)?;
    items::validate_intrinsic_location(ref_, intrinsic_span, state)?;
    state.with_param_constness(ParamConstness::ExplicitOnly, |state| {
        params::validate_params(&fn_.params, intrinsic_span.is_some(), state)?;
        validate_fn_return_type(fn_, state)?;
        Ok(())
    })?;
    validate_unique_signature(fn_, state);
    validate_unary_operator(fn_, state)?;
    validate_binary_operator(fn_, state)?;
    validate_body(fn_, state)?;
    validate_fn_name(fn_, state);
    items::validate_usage(ref_, state);
    Ok(())
}

fn validate_fn_name(fn_: &FnDefinition, state: &mut ValidateState<'_, '_>) {
    let allowed_cases = naming::fn_allowed_cases(fn_, state);
    naming::validate_name(fn_.name_span, allowed_cases, state);
}

fn validate_fn_return_type<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let (Some(arrow_span), Some(return_type)) = (fn_.arrow_span, &fn_.return_type) else {
        return Ok(());
    };
    state.with_const_mark_span(Some(arrow_span), |state| {
        exprs::validate_expr(return_type, state)
    })?;
    let actual_type = types::expr_type(return_type, state.inner);
    let expected_type = Type::Struct(state.inner.search_prelude_type("typeref"));
    exprs::validate_type_match(return_type.span(), actual_type, None, expected_type, state)?;
    Ok(())
}

fn validate_body<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let param_constness = if fn_.const_keyword_span.is_some() {
        ParamConstness::All
    } else {
        ParamConstness::ExplicitOnly
    };
    state.with_param_constness(param_constness, |state| {
        statements::validate_fn_statements(fn_, state)
    })?;
    Ok(())
}

fn validate_unique_signature<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) {
    if let Some(previous_fn) = find_previous_same_fn_signature(fn_, state) {
        let fn_key = key_rendering::fn_key(fn_, state.inner)
            .unwrap_or_else(|_| unreachable!("function should be validated before"));
        state.add_log(logs::items::duplicate_fn(
            &fn_key,
            fn_.signature_span_without_return,
            previous_fn.signature_span_without_return,
            state,
        ));
    }
}

fn validate_unary_operator(
    fn_: &FnDefinition,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if !UNARY_FN_NAMES.contains(&fn_.name.as_str()) {
        Ok(())
    } else if fn_.params.params.len() != 1 {
        let fn_key = key_rendering::fn_key(fn_, state.inner)?;
        state.add_log(logs::operators::unary_operator_param_count(
            &fn_key,
            fn_.signature_span_with_return,
            state,
        ));
        Err(ValidateError)
    } else if fn_.return_type.is_none() {
        let fn_key = key_rendering::fn_key(fn_, state.inner)?;
        state.add_log(logs::operators::unary_operator_without_return_type(
            &fn_key,
            fn_.signature_span_with_return,
            state,
        ));
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn validate_binary_operator(
    fn_: &FnDefinition,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if !BINARY_FN_NAMES.contains(&fn_.name.as_str()) {
        Ok(())
    } else if fn_.params.params.len() != 2 {
        let fn_key = key_rendering::fn_key(fn_, state.inner)?;
        state.add_log(logs::operators::binary_operator_param_count(
            &fn_key,
            fn_.signature_span_with_return,
            state,
        ));
        Err(ValidateError)
    } else if fn_.return_type.is_none() {
        let fn_key = key_rendering::fn_key(fn_, state.inner)?;
        state.add_log(logs::operators::binary_operator_without_return_type(
            &fn_key,
            fn_.signature_span_with_return,
            state,
        ));
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn find_previous_same_fn_signature<'item>(
    fn_: &'item FnDefinition,
    state: &ValidateState<'_, 'item>,
) -> Option<&'item FnDefinition> {
    let search_params = SearchParams {
        key: &fn_.key(),
        location: ItemRef::Fn(fn_),
        imports: &state.inner.imports,
        config: SearchConfig {
            can_be_after: false,
            can_be_parent_node: false,
        },
    };
    state
        .inner
        .items
        .search_in_same_file(search_params, Visibility::Enforced)
        .map(|item| match item {
            ItemRef::Fn(previous_fn) => previous_fn,
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("only functions are searched with parameter types")
            }
        })
        .find(|previous_fn| queries::fns::are_same_signatures(fn_, previous_fn, state.inner))
}
