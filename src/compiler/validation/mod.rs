mod exprs;
mod fns;
mod validators;

use crate::compiler::consts::ConstChecker;
use crate::compiler::dependencies::{DependencyResolver, DependencyType};
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::imports::{Import, ImportSegment};
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::parsing::modules::Module;
use crate::compiler::types::{Type, TypeResolver};
use crate::compiler::validation::validators::ident::Case;
use crate::utils::indexing::ItemNodeRef;
use crate::utils::parsing::span::Span;
use crate::utils::reading::ReadFile;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogLevel};
use std::path::Path;

pub(crate) struct Validator<'item, 'index> {
    pub(crate) context: ValidateContext<'index>,
    indexes: &'index Indexes<'item>,
    const_mark_span: Option<Span>,
    type_resolver: TypeResolver<'item, 'index>,
    key_renderer: KeyRenderer<'item, 'index>,
    const_checker: ConstChecker<'item, 'index>,
}

impl<'item, 'index> Validator<'item, 'index> {
    pub(crate) fn new(
        files: &'index [ReadFile],
        root_path: &'index Path,
        indexes: &'index Indexes<'item>,
    ) -> Self {
        Self {
            context: ValidateContext::new(files, root_path),
            indexes,
            const_mark_span: None,
            type_resolver: TypeResolver::new(indexes),
            key_renderer: KeyRenderer::new(indexes),
            const_checker: ConstChecker::new(indexes),
        }
    }

    pub(crate) fn validate_modules(
        mut self,
        modules: &[Module],
        is_warning_treated_as_error: bool,
    ) -> Result<Vec<Log>, Vec<Log>> {
        for module in modules {
            _ = self.validate_module(module);
        }
        if self
            .context
            .logs
            .iter()
            .any(|log| Self::is_log_error(log, is_warning_treated_as_error))
        {
            Err(self.context.logs)
        } else {
            Ok(self.context.logs)
        }
    }

    fn is_log_error(log: &Log, is_warning_treated_as_error: bool) -> bool {
        log.level == LogLevel::Error
            || (is_warning_treated_as_error && log.level == LogLevel::Warning)
    }

    fn validate_module(&mut self, node: &Module) -> Result<(), ValidateError> {
        let mut is_module_valid = true;
        let mut are_imports_finished = false;
        for item in &node.items {
            if let Item::Import(import) = item {
                if self.validate_import(import, !are_imports_finished).is_err() {
                    is_module_valid = false;
                }
            } else {
                are_imports_finished = true;
            }
        }
        if !is_module_valid {
            return Err(ValidateError);
        }
        for item in &node.items {
            _ = self.validate_item(item);
        }
        Ok(())
    }

    fn validate_import(&mut self, node: &Import, is_top_import: bool) -> Result<(), ValidateError> {
        let is_found = node.imported_file_index.is_some();
        let is_pub = node.pub_keyword_span.is_some();
        validators::import::check_found(is_found, &node.segments, &mut self.context)?;
        validators::import::check_top(is_top_import, node.span, &mut self.context)?;
        validators::import::check_self_import(
            node.imported_file_index,
            node.span,
            &mut self.context,
        );
        validators::import::check_usage(
            node.id,
            node.imported_file_index,
            node.span,
            is_pub,
            &node.segments,
            &mut self.context,
            self.indexes,
        );
        for &segment in &node.segments {
            if let ImportSegment::Name(span) = segment {
                validators::ident::check_case(span, &[Case::Snake], &mut self.context);
            }
        }
        Ok(())
    }

    fn validate_item(&mut self, node: &Item) -> Result<(), ValidateError> {
        match node {
            Item::Import(_) => Ok(()), // validated during previous pass
            Item::Var(item) => self.validate_var(item),
            Item::Const(item) => self.validate_const(item),
            Item::Struct(item) => self.validate_struct(item),
            Item::Fn(item) => self.validate_fn(item),
            Item::Repeat(item) => self.validate_repeat(item),
        }
    }

    fn validate_var(&mut self, node: &VarDefinition) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Var(node);
        let mut dependency_resolver =
            DependencyResolver::new(DependencyType::CycleDetection, self.indexes);
        let dependency_result = dependency_resolver.scan_var(node);
        validators::item::check_circular_dependencies(ref_, dependency_result, &mut self.context)?;
        validators::item::check_unique_definition(ref_, &mut self.context, self.indexes)?;
        validators::item::check_usage(ref_, &ref_.key(), &mut self.context, self.indexes);
        validators::ident::check_char_count(node.name_span, &mut self.context);
        validators::ident::check_case(node.name_span, &[Case::Snake], &mut self.context);
        self.validate_expr(&node.default_value, None)?;
        Ok(())
    }

    fn validate_const(&mut self, node: &ConstDefinition) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Const(node);
        let mut dependency_resolver =
            DependencyResolver::new(DependencyType::CycleDetection, self.indexes);
        let dependency_result = dependency_resolver.scan_const(node);
        validators::item::check_circular_dependencies(ref_, dependency_result, &mut self.context)?;
        validators::item::check_unique_definition(ref_, &mut self.context, self.indexes)?;
        validators::item::check_usage(ref_, &ref_.key(), &mut self.context, self.indexes);
        validators::ident::check_char_count(node.name_span, &mut self.context);
        let allowed_cases = self.const_allowed_cases(node);
        validators::ident::check_case(node.name_span, allowed_cases, &mut self.context);
        self.validate_expr(&node.value, Some(node.const_keyword_span))?;
        Ok(())
    }

    fn const_allowed_cases(&self, node: &ConstDefinition) -> &'static [Case] {
        let may_return_typeref = self
            .type_resolver
            .expr_type(&node.value)
            .struct_ref()
            .is_none_or(|type_| type_ == self.indexes.search_prelude_type("typeref"));
        if may_return_typeref {
            &[Case::ScreamingSnake, Case::Pascal]
        } else {
            &[Case::ScreamingSnake]
        }
    }

    fn validate_struct(&mut self, node: &StructDefinition) -> Result<(), ValidateError> {
        validators::item::check_prelude_location(ItemRef::Struct(node), &mut self.context)?;
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

    fn validate_params(&mut self, params: &ParamGroup) -> Result<(), ValidateError> {
        let mut are_params_valid = true;
        for param in &params.params {
            if self.validate_param(param).is_err() {
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

    fn validate_param(&mut self, param: &Param) -> Result<(), ValidateError> {
        self.validate_expr(&param.type_, Some(param.colon_span))?;
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
