use crate::compiler::consts::{self, ConstValue};
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::state::State;

pub(crate) fn is_const_infinite_f32(call: &Call, state: &State<'_>) -> bool {
    matches!(
        consts::call_value(call, state),
        ConstValue::F32(value) if !value.0.is_finite()
    )
}
