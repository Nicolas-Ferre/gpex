use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::vars::ConstDefinition;
use crate::compiler::state::{CompilerImplType, State};
use crate::compiler::validation::validators::ident::Case;
use crate::compiler::values::types;
use crate::compiler::values::types::Type;

pub(super) const IMPORT_ALLOWED_CASES: &[Case] = &[Case::Snake];
pub(super) const VAR_ALLOWED_CASES: &[Case] = &[Case::Snake];

pub(super) fn const_allowed_cases(
    node: &ConstDefinition,
    state: &mut State<'_>,
) -> &'static [Case] {
    let type_ = types::expr_type(&node.value, state);
    let may_be_typeref = type_.struct_ref().is_none()
        || state.is_compilerimpl_type(type_, CompilerImplType::Typeref);
    if may_be_typeref {
        &[Case::ScreamingSnake, Case::Pascal]
    } else {
        &[Case::ScreamingSnake]
    }
}

pub(super) fn fn_allowed_cases(node: &FnDefinition, state: &mut State<'_>) -> &'static [Case] {
    let type_ = types::fn_type(node, state);
    let may_return_typeref = match type_ {
        Type::Struct(_) => state.is_compilerimpl_type(type_, CompilerImplType::Typeref),
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
    let type_ = types::param_type(node, state);
    let is_typeref = state.is_compilerimpl_type(type_, CompilerImplType::Typeref);
    if is_typeref && node.const_mark_span().is_some() {
        &[Case::Snake, Case::Pascal]
    } else {
        &[Case::Snake]
    }
}
