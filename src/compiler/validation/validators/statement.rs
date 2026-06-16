use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::statements::{ReturnStatement, Statement};
use crate::utils::parsing::span::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_return_before_end(
    return_span: Span,
    next_statement_span: Span,
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
            msg: "`return` statement not at the end of the block".into(),
            location: Some(context.location(return_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "this statement is after".into(),
                location: Some(context.location(next_statement_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_missing_return<'statement>(
    statements: &'statement [Statement],
    previous_statement_span: Span,
    block_end_span: Span,
    return_type_span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<&'statement ReturnStatement, ValidateError> {
    if let Some(Statement::Return(return_statement)) = statements.last() {
        Ok(return_statement)
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "missing `return` statement".into(),
            location: Some(context.location(previous_statement_span.until(block_end_span))),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "function has a return type".into(),
                location: Some(context.location(return_type_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_disallowed_return(
    statements: &[Statement],
    fn_: &FnDefinition,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let mut result = Ok(());
    for statement in statements {
        if let Statement::Return(return_statement) = statement {
            context.logs.push(Log {
                level: LogLevel::Error,
                msg: "`return` statement in function with no return type".into(),
                location: Some(context.location(return_statement.span)),
                inner: vec![LogInner {
                    level: LogLevel::Info,
                    msg: "function has no return type".into(),
                    location: Some(context.location(fn_.signature_span_with_return)),
                }],
            });
            result = Err(ValidateError);
        }
    }
    result
}

pub(crate) fn check_empty_block(
    statements: &[Statement],
    body_span: Span,
    context: &mut ValidateContext<'_>,
) {
    if statements.is_empty() {
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: "empty statement block".into(),
            location: Some(context.location(body_span)),
            inner: vec![],
        });
    }
}
