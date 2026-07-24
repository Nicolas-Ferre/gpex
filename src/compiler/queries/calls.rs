use crate::compiler::consts::{self, ConstValue};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::CompilerImplFn;
use crate::compiler::state::State;

pub(crate) fn is_compilerimpl(call: &Call, fn_: CompilerImplFn, state: &State<'_>) -> bool {
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source)) if source.compilerimpl() == Some(fn_)
    )
}

pub(crate) fn is_const_infinite_f32(call: &Call, state: &State<'_>) -> bool {
    matches!(
        consts::call_value(call, state),
        ConstValue::F32(value) if !value.0.is_finite()
    )
}
