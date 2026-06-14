#![expect(clippy::multiple_inherent_impl)]

mod exprs;
mod fns;
mod items;
mod naming;
mod validators;

use crate::compiler::consts::{ConstChecker, ParamConstness};
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::imports::{Import, ImportSegment};
use crate::compiler::parsing::modules::Module;
use crate::compiler::values::ValueResolver;
use crate::utils::parsing::span::Span;
use crate::utils::reading::ReadFile;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogLevel};
use std::path::Path;

#[derive(Debug)]
pub(crate) struct Validator<'item, 'index> {
    pub(crate) context: ValidateContext<'index>,
    indexes: &'index Indexes<'item>,
    const_mark_span: Option<Span>,
    value_resolver: ValueResolver<'item, 'index>,
    key_renderer: KeyRenderer<'item, 'index>,
    const_checker: ConstChecker,
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
            value_resolver: ValueResolver::new(indexes),
            key_renderer: KeyRenderer::new(indexes),
            const_checker: ConstChecker::new(),
        }
    }

    pub(crate) fn validate_modules(
        mut self,
        modules: &'item [Module],
        is_warning_treated_as_error: bool,
    ) -> Result<Vec<Log>, Vec<Log>> {
        let mut is_error_detected = false;
        for module in modules {
            is_error_detected |= self.validate_module(module).is_err();
        }
        if !is_error_detected {
            // Import usage is checked in dedicated pass to avoid false positive
            // in case of cyclic dependencies.
            for module in modules {
                self.validate_module_import_usage(module);
            }
        }
        self.context.logs.sort_by_key(Log::sort_key);
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

    fn validate_module(&mut self, node: &'item Module) -> Result<(), ValidateError> {
        let mut is_module_valid = true;
        let mut are_imports_finished = false;
        for item in &node.items {
            if let Item::Import(import) = item {
                is_module_valid &= self.validate_import(import, !are_imports_finished).is_ok();
            } else {
                are_imports_finished = true;
            }
        }
        if !is_module_valid {
            return Err(ValidateError);
        }
        for item in &node.items {
            is_module_valid &= self.validate_item(item).is_ok();
        }
        if !is_module_valid {
            return Err(ValidateError);
        }
        Ok(())
    }

    fn validate_import(&mut self, node: &Import, is_top_import: bool) -> Result<(), ValidateError> {
        let is_found = node.imported_file_index.is_some();
        validators::import::check_found(is_found, &node.segments, &mut self.context)?;
        validators::import::check_top(is_top_import, node.span, &mut self.context)?;
        validators::import::check_self_import(
            node.imported_file_index,
            node.span,
            &mut self.context,
        );
        for &segment in &node.segments {
            if let ImportSegment::Name(span) = segment {
                validators::ident::check_case(span, Self::IMPORT_ALLOWED_CASES, &mut self.context);
            }
        }
        Ok(())
    }

    fn validate_module_import_usage(&mut self, node: &Module) {
        for item in &node.items {
            if let Item::Import(import) = item {
                validators::import::check_usage(
                    import.id,
                    import.imported_file_index,
                    import.span,
                    import.pub_keyword_span.is_some(),
                    &import.segments,
                    &mut self.context,
                    self.indexes,
                );
            }
        }
    }

    fn run_with_param_constness<O>(
        &mut self,
        param_constness: ParamConstness,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let previous_param_constness = self.const_checker.param_constness;
        self.const_checker.param_constness = param_constness;
        let result = callback(self);
        self.const_checker.param_constness = previous_param_constness;
        result
    }

    fn with_const_mark_span<O>(
        &mut self,
        span: Option<Span>,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let previous_const_mark_span = self.const_mark_span;
        self.const_mark_span = span;
        let output = callback(self);
        self.const_mark_span = previous_const_mark_span;
        output
    }
}
