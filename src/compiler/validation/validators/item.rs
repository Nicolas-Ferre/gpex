use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::calls::UNARY_FN_NAMES;
use crate::compiler::parsing::exprs::{BINARY_FN_NAMES, OPERATOR_FN_NAME_PREFIX};
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::validation::{ValidateState, logs};
use crate::compiler::values::types;
use crate::compiler::values::types::Type;
use crate::utils::indexing::{ItemNodeRef, NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::ValidateError;

pub(crate) fn check_circular_dependencies(
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

pub(crate) fn check_unique_definition<'item>(
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

pub(crate) fn check_unique_params(
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

pub(crate) fn check_unique_fn_signature<'item>(
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

pub(crate) fn check_prelude_location(
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

pub(crate) fn check_usage<'item>(item: ItemRef<'item>, state: &mut ValidateState<'_, 'item>) {
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

pub(crate) fn check_found<'item>(
    source: Option<ItemRef<'item>>,
    node: impl NodeRef,
    span: Span,
    key: &str,
    displayed_key: &str,
    state: &mut ValidateState<'_, 'item>,
) -> Result<ItemRef<'item>, ValidateError> {
    if let Some(source) = source {
        Ok(source)
    } else {
        let log = if let Some(candidates) = state.inner.candidate_sources.get(&node.id()) {
            logs::items::not_found_with_similar_candidates(
                displayed_key,
                span,
                candidates
                    .iter()
                    .map(|candidate| candidate.signature_span_with_return()),
                state,
            )
        } else if let Some(priv_source) = state.inner.priv_sources.get(&node.id()) {
            logs::items::not_found_with_priv_candidate(
                displayed_key,
                span,
                priv_source.name_span(),
                state,
            )
        } else {
            logs::items::not_found_with_importable_candidate(
                displayed_key,
                span,
                state
                    .inner
                    .items
                    .iter_by_key(key)
                    .filter(ItemNodeRef::is_pub)
                    .map(|item| (state.context.dot_path(item.file_index()), item.name_span())),
                state,
            )
        };
        state.add_log(log);
        Err(ValidateError)
    }
}

pub(crate) fn check_unary_operator_fn(
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

pub(crate) fn check_binary_operator_fn(
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
        .find(|previous_fn| are_same_fn_signatures(fn_, previous_fn, state))
}

fn are_same_fn_signatures<'item>(
    fn_: &'item FnDefinition,
    other_fn: &'item FnDefinition,
    state: &ValidateState<'_, 'item>,
) -> bool {
    debug_assert!(fn_.name == other_fn.name);
    debug_assert!(fn_.params.params.len() == other_fn.params.params.len());
    if fn_.has_requirement() || other_fn.has_requirement() {
        return false;
    }
    fn_.params
        .params
        .iter()
        .zip(&other_fn.params.params)
        .all(|(param, other_param)| {
            let type_ = types::param_type(param, state.inner);
            let other_type = types::param_type(other_param, state.inner);
            are_same_param_types(type_, other_type, fn_, other_fn)
        })
}

fn are_same_param_types(
    type_: Type<'_>,
    other_type: Type<'_>,
    fn_: &FnDefinition,
    other_fn: &FnDefinition,
) -> bool {
    match (type_, other_type) {
        (Type::Struct(struct_), Type::Struct(other_struct)) => struct_.id == other_struct.id,
        (Type::Param(param), Type::Param(other_param))
        | (Type::Wildcard(param), Type::Wildcard(other_param)) => {
            param_index(fn_, param) == param_index(other_fn, other_param)
        }
        _ => false,
    }
}

fn param_index(fn_: &FnDefinition, param: &Param) -> usize {
    fn_.params
        .params
        .iter()
        .position(|fn_param| fn_param.id == param.id)
        .unwrap_or_else(|| unreachable!("param should be found in the function"))
}
