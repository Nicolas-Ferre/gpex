use crate::compiler::dependencies;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::BINARY_FN_NAMES;
use crate::compiler::parsing::exprs::calls::UNARY_FN_NAMES;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::statements::{AssignmentStatement, ReturnStatement, Statement};
use crate::compiler::refs;
use crate::compiler::validation::{ParamConstness, ValidateState, exprs, items, logs, naming};
use crate::compiler::values::types;
use crate::compiler::values::types::Type;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

// TODO: split file

pub(super) fn validate_fn<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Fn(fn_);
    let compilerimpl_span = fn_.body.compilerimpl_keyword_span();
    let mut dependencies = Dependencies::new();
    let dependency_result = dependencies::scan_fn(fn_, &mut dependencies, state.inner);
    items::validate_no_circular_dependencies(ref_, dependency_result, state)?;
    items::validate_compilerimpl_location(ref_, compilerimpl_span, state)?;
    state.with_param_constness(ParamConstness::ExplicitOnly, |state| {
        items::validate_params(&fn_.params, compilerimpl_span.is_some(), state)?;
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
    naming::validate_case(fn_.name_span, allowed_cases, state);
    naming::validate_char_count(fn_.name_span, state);
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
    state.with_param_constness(param_constness, |state| validate_fn_statements(fn_, state))?;
    Ok(())
}

fn validate_fn_statements<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let FnBody::Statements(body) = &fn_.body else {
        return Ok(());
    };
    let mut is_error_detected = false;
    state.with_const_mark_span(fn_.const_keyword_span, |state| {
        for (index, statement) in body.statements.iter().enumerate() {
            is_error_detected |= validate_statement(statement, state).is_err();
            if let Statement::Return(return_) = statement {
                let next_statement_span = body
                    .statements
                    .get(index + 1)
                    .map_or(body.body_end_span, Statement::span);
                is_error_detected |= validate_return_position(
                    return_.span,
                    next_statement_span,
                    index,
                    body.statements.len(),
                    state,
                )
                .is_err();
            }
        }
    });
    if is_error_detected {
        return Err(ValidateError);
    }
    if let Some(return_type) = &fn_.return_type {
        let previous_statement_span = body
            .statements
            .last()
            .map_or(body.body_start_span, Statement::span);
        let return_statement = validate_required_return(
            &body.statements,
            previous_statement_span,
            body.body_end_span,
            return_type.span(),
            state,
        )?;
        let actual_type = types::expr_type(&return_statement.value, state.inner);
        let expected_type = types::fn_type(fn_, state.inner);
        exprs::validate_type_match(
            return_statement.value.span(),
            actual_type,
            Some(return_type.span()),
            expected_type,
            state,
        )?;
    } else {
        validate_disallowed_returns(&body.statements, fn_, state)?;
        if body.statements.is_empty() {
            state.add_log(logs::statements::empty_block(body.body_span, state));
        }
    }
    Ok(())
}

fn validate_statement(
    statement: &Statement,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    match statement {
        Statement::Return(return_) => exprs::validate_expr(&return_.value, state),
        Statement::Assignment(assignment) => validate_assignment_statement(assignment, state),
    }
}

fn validate_assignment_statement(
    assignment: &AssignmentStatement,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let assigned_result = validate_assignment_statement_assigned(assignment, state);
    let value_result = validate_assignment_statement_value(assignment, state);
    assigned_result.and(value_result)
}

fn validate_assignment_statement_assigned(
    assignment: &AssignmentStatement,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    exprs::validate_expr(&assignment.assigned, state)?;
    validate_assignment_target(&assignment.assigned, state);
    Ok(())
}

fn validate_assignment_statement_value(
    assignment: &AssignmentStatement,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    exprs::validate_expr(&assignment.value, state)?;
    let actual_type = types::expr_type(&assignment.value, state.inner);
    let expected_type = types::expr_type(&assignment.assigned, state.inner);
    exprs::validate_type_match(
        assignment.value.span(),
        actual_type,
        Some(assignment.assigned.span()),
        expected_type,
        state,
    )?;
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

fn validate_return_position(
    return_span: Span,
    next_statement_span: Span,
    position: usize,
    statement_count: usize,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    debug_assert_ne!(statement_count, 0);
    if position == statement_count - 1 {
        Ok(())
    } else {
        state.add_log(logs::statements::return_before_end(
            return_span,
            next_statement_span,
            state,
        ));
        Err(ValidateError)
    }
}

fn validate_required_return<'statement>(
    statements: &'statement [Statement],
    previous_statement_span: Span,
    block_end_span: Span,
    return_type_span: Span,
    state: &mut ValidateState<'_, '_>,
) -> Result<&'statement ReturnStatement, ValidateError> {
    if let Some(Statement::Return(return_statement)) = statements.last() {
        Ok(return_statement)
    } else {
        state.add_log(logs::statements::missing_return(
            previous_statement_span.until(block_end_span),
            return_type_span,
            state,
        ));
        Err(ValidateError)
    }
}

fn validate_disallowed_returns(
    statements: &[Statement],
    fn_: &FnDefinition,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let mut result = Ok(());
    for statement in statements {
        if let Statement::Return(return_statement) = statement {
            state.add_log(logs::statements::disallowed_return(
                return_statement.span,
                fn_.signature_span_with_return,
                state,
            ));
            result = Err(ValidateError);
        }
    }
    result
}

fn validate_assignment_target(
    expr: &crate::compiler::parsing::exprs::Expr,
    state: &mut ValidateState<'_, '_>,
) {
    if refs::is_expr_ref(expr, state.inner) == Some(false) {
        state.add_log(logs::exprs::not_ref(expr.span(), state));
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
