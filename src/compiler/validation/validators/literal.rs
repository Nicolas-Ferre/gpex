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
    if is_value_valid {
        Ok(())
    } else {
        state.add_log(Log {
            level: LogLevel::Error,
            msg: format!("`{type_name}` literal out of bounds"),
            location: Some(state.span_location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}
