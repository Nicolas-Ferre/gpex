use crate::compiler::validation::{ValidateState, logs};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

pub(crate) fn check_bounds(
    is_value_valid: bool,
    span: Span,
    type_name: &str,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if is_value_valid {
        Ok(())
    } else {
        state.add_log(logs::exprs::literal_out_of_bounds(type_name, span, state));
        Err(ValidateError)
    }
}
