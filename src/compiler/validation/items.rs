use crate::compiler::dependencies::DependencyResolver;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::types::Type;
use crate::compiler::validation::{Validator, validators};
use crate::utils::indexing::ItemNodeRef;
use crate::utils::validation::ValidateError;

impl Validator<'_, '_> {
    pub(crate) fn validate_item(&mut self, node: &Item) -> Result<(), ValidateError> {
        debug_assert!(self.const_mark_span.is_none());
        match node {
            Item::Import(_) => Ok(()), // validated during previous pass
            Item::Var(item) => self.validate_var(item),
            Item::Const(item) => self.validate_const(item),
            Item::Struct(item) => self.validate_struct(item),
            Item::Fn(item) => self.validate_fn(item),
            Item::Repeat(item) => self.validate_repeat(item),
        }
    }

    pub(crate) fn validate_params(
        &mut self,
        params: &ParamGroup,
        is_compilerimpl: bool,
    ) -> Result<(), ValidateError> {
        let mut are_params_valid = true;
        for param in &params.params {
            if self.validate_param(param, is_compilerimpl).is_err() {
                are_params_valid = false;
            }
        }
        validators::item::check_unique_params(&params.params, &mut self.context)?;
        if are_params_valid {
            Ok(())
        } else {
            Err(ValidateError)
        }
    }

    fn validate_var(&mut self, node: &VarDefinition) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Var(node);
        let dependency_result = DependencyResolver::new(self.indexes).scan_var(node);
        validators::item::check_circular_dependencies(ref_, dependency_result, &mut self.context)?;
        validators::item::check_unique_definition(ref_, &mut self.context, self.indexes)?;
        validators::item::check_usage(ref_, &ref_.key(), &mut self.context, self.indexes);
        validators::ident::check_char_count(node.name_span, &mut self.context);
        validators::ident::check_case(node.name_span, Self::VAR_ALLOWED_CASES, &mut self.context);
        self.validate_expr(&node.default_value)?;
        Ok(())
    }

    fn validate_const(&mut self, node: &ConstDefinition) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Const(node);
        let dependency_result = DependencyResolver::new(self.indexes).scan_const(node);
        validators::item::check_circular_dependencies(ref_, dependency_result, &mut self.context)?;
        validators::item::check_unique_definition(ref_, &mut self.context, self.indexes)?;
        validators::item::check_usage(ref_, &ref_.key(), &mut self.context, self.indexes);
        validators::ident::check_char_count(node.name_span, &mut self.context);
        let allowed_cases = self.const_allowed_cases(node);
        validators::ident::check_case(node.name_span, allowed_cases, &mut self.context);
        self.with_const_mark_span(Some(node.const_keyword_span), |self_| {
            self_.validate_expr(&node.value)
        })?;
        Ok(())
    }

    fn validate_struct(&mut self, node: &StructDefinition) -> Result<(), ValidateError> {
        validators::item::check_prelude_location(
            ItemRef::Struct(node),
            Some(node.compilerimpl_keyword_span),
            &mut self.context,
        )?;
        Ok(())
    }

    fn validate_repeat(&mut self, node: &RepeatDefinition) -> Result<(), ValidateError> {
        self.validate_call(&node.call)?;
        validators::expr::check_has_return_type(
            &node.call,
            node.call.span,
            &mut self.context,
            self.indexes,
        )?;
        Ok(())
    }

    fn validate_param(
        &mut self,
        param: &Param,
        is_compilerimpl: bool,
    ) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Param(param);
        self.validate_param_type(param)?;
        if !is_compilerimpl {
            validators::item::check_usage(ref_, &ref_.key(), &mut self.context, self.indexes);
        }
        validators::ident::check_char_count(param.name_span, &mut self.context);
        let allowed_cases = self.param_allowed_cases(param);
        validators::ident::check_case(param.name_span, allowed_cases, &mut self.context);
        Ok(())
    }

    fn validate_param_type(&mut self, param: &Param) -> Result<(), ValidateError> {
        if matches!(param.type_, Expr::Wildcard(_)) {
            return Ok(());
        };
        self.with_const_mark_span(Some(param.colon_span), |self_| {
            self_.validate_expr(&param.type_)
        })?;
        validators::expr::check_types(
            param.type_.span(),
            self.type_resolver.expr_type(&param.type_),
            None,
            Type::Struct(self.indexes.search_prelude_type("typeref")),
            &mut self.context,
        )?;
        Ok(())
    }
}
