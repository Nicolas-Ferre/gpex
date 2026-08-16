use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::{IntrinsicType, State};
use crate::compiler::types;

pub(crate) fn is_const_typeref<'item>(param: &'item Param, state: &State<'item>) -> bool {
    param.const_mark_span().is_some()
        && state.is_intrinsic_type(types::param_type(param, state), IntrinsicType::Typeref)
}
