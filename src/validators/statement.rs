use crate::language::statements::Statement;
use crate::language::statements::return_::ReturnStatement;
use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_return_before_end(
    span: Span,
    position: usize,
    statement_count: usize,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    debug_assert_ne!(statement_count, 0);
    if position == statement_count - 1 {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "`return` statement not at the end of the block".into(),
            location: Some(context.location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_missing_return<'statement>(
    statements: &'statement [Statement],
    block_end_span: Span,
    return_type_span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<&'statement ReturnStatement, ValidateError> {
    if let Some(Statement::Return(return_statement)) = statements.last() {
        Ok(return_statement)
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "missing `return` statement".into(),
            location: Some(context.location(block_end_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "function has a return type".into(),
                location: Some(context.location(return_type_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_disallowed_return(
    statements: &[Statement],
    function_name_span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let mut result = Ok(());
    for statement in statements {
        if let Statement::Return(return_statement) = statement {
            context.logs.push(Log {
                level: LogLevel::Error,
                message: "`return` statement in function with no return type".into(),
                location: Some(context.location(return_statement.span)),
                inner: vec![LogInner {
                    level: LogLevel::Info,
                    message: "function has no return type".into(),
                    location: Some(context.location(function_name_span)),
                }],
            });
            result = Err(ValidateError);
        }
    }
    result
}

pub(crate) fn check_empty_body(
    statements: &[Statement],
    body_end_span: Span,
    context: &mut ValidateContext<'_>,
) {
    if statements.is_empty() {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: "function body is empty".into(),
            location: Some(context.location(body_end_span)),
            inner: vec![],
        });
    }
}
