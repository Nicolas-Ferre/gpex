use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::vars::ConstDefinition;
use crate::compiler::state::State;
use crate::compiler::validation::validators::ident::Case;
use crate::compiler::values::types;
use crate::compiler::values::types::Type;

pub(super) const IMPORT_ALLOWED_CASES: &[Case] = &[Case::Snake];
pub(super) const VAR_ALLOWED_CASES: &[Case] = &[Case::Snake];

pub(super) fn const_allowed_cases(
    node: &ConstDefinition,
    state: &mut State<'_>,
) -> &'static [Case] {
    let typeref_type = state.search_prelude_type("typeref");
    let may_be_typeref = types::expr_type(&node.value, state)
        .struct_ref()
        .is_none_or(|type_| type_ == typeref_type);
    if may_be_typeref {
        &[Case::ScreamingSnake, Case::Pascal]
    } else {
        &[Case::ScreamingSnake]
    }
}

pub(super) fn fn_allowed_cases(node: &FnDefinition, state: &mut State<'_>) -> &'static [Case] {
    let typeref_type = state.search_prelude_type("typeref");
    let may_return_typeref = match types::fn_type(node, state) {
        Type::Struct(struct_ref) => struct_ref == typeref_type,
        Type::Param(_) | Type::Wildcard(_) | Type::NoReturn => false,
        Type::Unknown => unreachable!("return type should be validated before"),
    };
    if may_return_typeref && node.const_keyword_span.is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}

pub(super) fn param_allowed_cases<'item>(
    node: &'item Param,
    state: &mut State<'item>,
) -> &'static [Case] {
    let typeref_type = state.search_prelude_type("typeref");
    let is_typeref = types::param_type(node, state).struct_ref() == Some(typeref_type);
    if is_typeref && node.const_mark_span().is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}
