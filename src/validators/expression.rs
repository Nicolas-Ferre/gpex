use crate::language::items::struct_::StructDefinition;
use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_types(
    actual_span: Span,
    actual_type: Option<&StructDefinition>,
    expected_span: Option<Span>,
    expected_type: Option<&StructDefinition>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if let Some(expected_type) = expected_type
        && let Some(actual_type) = actual_type
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
