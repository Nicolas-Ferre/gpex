use crate::compiler::transversal::ast::exprs::calls::Call;
use crate::compiler::transversal::ast::item_ref::ItemRef;
use crate::compiler::transversal::ast::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::transversal::consts::{self, ConstValue};
use crate::compiler::transversal::state::State;

pub(crate) fn is_intrinsic(call: &Call, fn_: IntrinsicFn, state: &State<'_>) -> bool {
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source)) if source.intrinsic() == Some(fn_)
    )
}

pub(crate) fn is_binary_intrinsic(call: &Call, fn_: BinaryIntrinsicFn, state: &State<'_>) -> bool {
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source)) if source.intrinsic() == Some(IntrinsicFn::Binary(fn_))
    )
}

pub(crate) fn is_const_infinite_f32(call: &Call, state: &State<'_>) -> bool {
    matches!(
        consts::call_value(call, state),
        ConstValue::F32(value) if !value.0.is_finite()
    )
}
