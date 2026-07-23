use crate::compiler::logs;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::statements::{ReturnStatement, Statement};
use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

pub(crate) fn check_return_before_end(
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

pub(crate) fn check_missing_return<'statement>(
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

pub(crate) fn check_disallowed_return(
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

pub(crate) fn check_empty_block(
    statements: &[Statement],
    body_span: Span,
    state: &mut ValidateState<'_, '_>,
) {
    if statements.is_empty() {
        state.add_log(logs::statements::empty_block(body_span, state));
    }
}
