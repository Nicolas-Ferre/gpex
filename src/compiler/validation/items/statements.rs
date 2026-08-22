use crate::compiler::parsing::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::statements::{AssignmentStatement, ReturnStatement, Statement};
use crate::compiler::refs;
use crate::compiler::types;
use crate::compiler::validation::{ValidateState, exprs, logs};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

pub(super) fn validate_fn_statements<'item>(
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let FnBody::Statements(body) = &fn_.body else {
        return Ok(());
    };
    validate_all_statements(body, fn_, state)?;
    validate_fn_return(body, fn_, state)
}

fn validate_all_statements<'item>(
    body: &FnStatementsBody,
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
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
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn validate_fn_return<'item>(
    body: &FnStatementsBody,
    fn_: &'item FnDefinition,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
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
        )
    } else {
        validate_disallowed_returns(&body.statements, fn_, state)?;
        if body.statements.is_empty() {
            state.add_log(logs::statements::empty_block(body.body_span, state));
        }
        Ok(())
    }
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
    let expr = &assignment.assigned;
    if refs::is_expr_ref(expr, state.inner) == Some(false) {
        state.add_log(logs::exprs::not_ref(expr.span(), state));
    }
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
