use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::state::State;
use crate::compiler::types::{self, Type};

pub(crate) fn are_same_signatures<'item>(
    fn_: &'item FnDefinition,
    other_fn: &'item FnDefinition,
    state: &State<'item>,
) -> bool {
    debug_assert!(fn_.name == other_fn.name);
    debug_assert!(fn_.params.params.len() == other_fn.params.params.len());
    if fn_.has_requirement() || other_fn.has_requirement() {
        return false;
    }
    fn_.params
        .params
        .iter()
        .zip(&other_fn.params.params)
        .all(|(param, other_param)| {
            let type_ = types::param_type(param, state);
            let other_type = types::param_type(other_param, state);
            are_same_param_types(type_, other_type, fn_, other_fn)
        })
}

fn are_same_param_types(
    type_: Type<'_>,
    other_type: Type<'_>,
    fn_: &FnDefinition,
    other_fn: &FnDefinition,
) -> bool {
    match (type_, other_type) {
        (Type::Struct(struct_), Type::Struct(other_struct)) => struct_.id == other_struct.id,
        (Type::Param(param), Type::Param(other_param))
        | (Type::Wildcard(param), Type::Wildcard(other_param)) => {
            param_index(fn_, param) == param_index(other_fn, other_param)
        }
        _ => false,
    }
}

fn param_index(fn_: &FnDefinition, param: &Param) -> usize {
    fn_.params
        .params
        .iter()
        .position(|fn_param| fn_param.id == param.id)
        .unwrap_or_else(|| unreachable!("param should be found in the function"))
}
