use crate::compiler::transversal::ast::exprs::calls::Call;
use crate::compiler::transversal::ast::items::ItemRef;
use crate::compiler::transversal::prelude::fns::{self, BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::transversal::state::State;
use crate::compiler::transversal::values::consts::{self, ConstValue};

pub(crate) fn is_intrinsic(call: &Call, fn_: IntrinsicFn, state: &State<'_>) -> bool {
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source)) if fns::intrinsic(source) == Some(fn_)
    )
}

pub(crate) fn is_binary_intrinsic(call: &Call, fn_: BinaryIntrinsicFn, state: &State<'_>) -> bool {
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source)) if fns::intrinsic(source) == Some(IntrinsicFn::Binary(fn_))
    )
}

pub(crate) fn is_const_infinite_f32(call: &Call, state: &State<'_>) -> bool {
    matches!(
        consts::call_value(call, state),
        ConstValue::F32(value) if !value.0.is_finite()
    )
}
