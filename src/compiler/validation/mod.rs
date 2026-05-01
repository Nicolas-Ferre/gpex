#![expect(clippy::multiple_inherent_impl)]

mod exprs;
mod fns;
mod items;
mod validators;

use crate::compiler::consts::ConstChecker;
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::imports::{Import, ImportSegment};
use crate::compiler::parsing::modules::Module;
use crate::compiler::types::TypeResolver;
use crate::compiler::validation::validators::ident::Case;
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
}
