use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::vars::ConstDefinition;
use crate::compiler::types::Type;
use crate::compiler::validation::Validator;
use crate::compiler::validation::validators::ident::Case;

impl<'item> Validator<'item, '_> {
    pub(super) const IMPORT_ALLOWED_CASES: &'static [Case] = &[Case::Snake];
    pub(super) const VAR_ALLOWED_CASES: &'static [Case] = &[Case::Snake];

    pub(super) fn const_allowed_cases(&mut self, node: &ConstDefinition) -> &'static [Case] {
        let typeref_type = self.indexes.search_prelude_type("typeref");
        let may_be_typeref = self
            .type_resolver
            .expr_type(&node.value)
            .struct_ref()
            .is_none_or(|type_| type_ == typeref_type);
        if may_be_typeref {
            &[Case::ScreamingSnake, Case::Pascal]
        } else {
            &[Case::ScreamingSnake]
        }
    }

    pub(super) fn fn_allowed_cases(&mut self, node: &FnDefinition) -> &'static [Case] {
        let typeref_type = self.indexes.search_prelude_type("typeref");
        let may_return_typeref = match self.type_resolver.fn_type(node) {
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

    pub(super) fn param_allowed_cases(&mut self, node: &'item Param) -> &'static [Case] {
        let typeref_type = self.indexes.search_prelude_type("typeref");
        let is_typeref = self.type_resolver.param_type(node).struct_ref() == Some(typeref_type);
        if is_typeref && node.const_mark_span().is_some() {
            &[Case::Snake, Case::Pascal]
        } else {
            &[Case::Snake]
        }
    }
}
