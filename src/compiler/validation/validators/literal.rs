use crate::compiler::state::State;
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;
use crate::{Log, LogLevel};

pub(crate) fn check_bounds(
    is_value_valid: bool,
    span: Span,
    type_name: &str,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    let context = &mut state.validation_context;
    if is_value_valid {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{type_name}` literal out of bounds"),
            location: Some(context.location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}
