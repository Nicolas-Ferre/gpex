use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_types(
    actual_span: Span,
    actual_type: Type<'_>,
    expected_span: Option<Span>,
    expected_type: Type<'_>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if let Type::Struct(expected_type) = expected_type
        && let Type::Struct(actual_type) = actual_type
    {
        if actual_type == expected_type {
            Ok(())
        } else {
            context.logs.push(Log {
                level: LogLevel::Error,
                message: format!("expression with invalid type `{}`", actual_type.name),
                location: Some(context.location(actual_span)),
                inner: vec![LogInner {
                    level: LogLevel::Info,
                    message: format!("expected `{}` type", expected_type.name),
                    location: expected_span.map(|span| context.location(span)),
                }],
            });
            Err(ValidateError)
        }
    } else {
        Err(ValidateError)
    }
}

pub(crate) fn check_constant(
    constant_value: Constant<'_>,
    span: Span,
    constant_mark_span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if constant_value == Constant::RuntimeValue {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "expression not constant".into(),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "expression must be constant".into(),
                location: Some(context.location(constant_mark_span)),
            }],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_no_return_type(
    node: impl NodeRef,
    span: Span,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if let Some(&ItemRef::Function(function)) = indexes.sources.get(&node.id())
        && function.return_type.is_none()
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("called function `{}` with no return type", function.key()),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "function has no return type".into(),
                location: Some(context.location(function.signature_span)),
            }],
        });
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_ref(
    expression: &Expression,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    if expression.is_ref(indexes) == Some(false) {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "expression is not a reference".into(),
            location: Some(context.location(expression.span())),
            inner: vec![],
        });
    }
}
