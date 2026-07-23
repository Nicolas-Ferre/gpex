use crate::compiler::dependencies;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::statements::{AssignmentStatement, Statement};
use crate::compiler::state::{ParamConstness, State};
use crate::compiler::validation::{exprs, items, naming, validators};
use crate::compiler::values::types;
use crate::compiler::values::types::Type;
use crate::utils::validation::ValidateError;

pub(super) fn validate_fn<'item>(
    node: &'item FnDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let ref_ = ItemRef::Fn(node);
    let compilerimpl_span = node.body.compilerimpl_keyword_span();
    let dependency_result = dependencies::scan_fn(node, state);
    validators::item::check_circular_dependencies(ref_, dependency_result, state)?;
    validators::item::check_prelude_location(ref_, compilerimpl_span, state)?;
    state.with_param_constness(ParamConstness::ExplicitOnly, |state| {
        items::validate_params(&node.params, compilerimpl_span.is_some(), state)?;
        validate_fn_return_type(node, state)?;
        Ok(())
    })?;
    validators::item::check_unique_fn_signature(node, state);
    validators::item::check_unary_operator_fn(node, state)?;
    validators::item::check_binary_operator_fn(node, state)?;
    validate_body(node, state)?;
    validate_fn_name(node, state);
    validators::item::check_usage(ref_, state);
    Ok(())
}

fn validate_fn_name(node: &FnDefinition, state: &mut State<'_>) {
    let allowed_cases = naming::fn_allowed_cases(node, state);
    validators::ident::check_case(node.name_span, allowed_cases, state);
    validators::ident::check_char_count(node.name_span, state);
}

fn validate_fn_return_type<'item>(
    node: &'item FnDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let (Some(arrow_span), Some(return_type)) = (node.arrow_span, &node.return_type) else {
        return Ok(());
    };
    state.with_const_mark_span(Some(arrow_span), |state| {
        exprs::validate_expr(return_type, state)
    })?;
    let actual_type = types::expr_type(return_type, state);
    let expected_type = Type::Struct(state.search_prelude_type("typeref"));
    validators::expr::check_types(return_type.span(), actual_type, None, expected_type, state)?;
    Ok(())
}

fn validate_body<'item>(
    node: &'item FnDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let param_constness = if node.const_keyword_span.is_some() {
        ParamConstness::All
    } else {
        ParamConstness::ExplicitOnly
    };
    state.with_param_constness(param_constness, |state| validate_fn_statements(node, state))?;
    Ok(())
}

fn validate_fn_statements<'item>(
    node: &'item FnDefinition,
    state: &mut State<'item>,
) -> Result<(), ValidateError> {
    let FnBody::Statements(body) = &node.body else {
        return Ok(());
    };
    let mut is_error_detected = false;
    state.with_const_mark_span(node.const_keyword_span, |state| {
        for (index, statement) in body.statements.iter().enumerate() {
            is_error_detected |= validate_statement(statement, state).is_err();
            if let Statement::Return(return_) = statement {
                let next_statement_span = body
                    .statements
                    .get(index + 1)
                    .map_or(body.body_end_span, Statement::span);
                is_error_detected |= validators::statement::check_return_before_end(
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
    if let Some(return_type) = &node.return_type {
        let previous_statement_span = body
            .statements
            .last()
            .map_or(body.body_start_span, Statement::span);
        let return_statement = validators::statement::check_missing_return(
            &body.statements,
            previous_statement_span,
            body.body_end_span,
            return_type.span(),
            state,
        )?;
        let actual_type = types::expr_type(&return_statement.value, state);
        let expected_type = types::fn_type(node, state);
        validators::expr::check_types(
            return_statement.value.span(),
            actual_type,
            Some(return_type.span()),
            expected_type,
            state,
        )?;
    } else {
        validators::statement::check_disallowed_return(&body.statements, node, state)?;
        validators::statement::check_empty_block(&body.statements, body.body_span, state);
    }
    Ok(())
}

fn validate_statement(node: &Statement, state: &mut State<'_>) -> Result<(), ValidateError> {
    match node {
        Statement::Return(statement) => exprs::validate_expr(&statement.value, state),
        Statement::Assignment(statement) => validate_assignment_statement(statement, state),
    }
}

fn validate_assignment_statement(
    node: &AssignmentStatement,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    let assigned_result = validate_assignment_statement_assigned(node, state);
    let value_result = validate_assignment_statement_value(node, state);
    assigned_result.and(value_result)
}

fn validate_assignment_statement_assigned(
    node: &AssignmentStatement,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    exprs::validate_expr(&node.assigned, state)?;
    validators::expr::check_ref(&node.assigned, state);
    Ok(())
}

fn validate_assignment_statement_value(
    node: &AssignmentStatement,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    exprs::validate_expr(&node.value, state)?;
    let actual_type = types::expr_type(&node.value, state);
    let expected_type = types::expr_type(&node.assigned, state);
    validators::expr::check_types(
        node.value.span(),
        actual_type,
        Some(node.assigned.span()),
        expected_type,
        state,
    )?;
    Ok(())
}
