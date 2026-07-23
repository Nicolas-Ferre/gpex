mod exprs;
mod fns;
mod items;
mod logs;
mod naming;
mod validators;

use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::imports::{Import, ImportSegment};
use crate::compiler::parsing::modules::Module;
use crate::compiler::state::State;
use crate::utils::parsing::span::Span;
use crate::utils::reading::ReadFile;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogLevel, LogLocation};
use std::mem;
use std::path::Path;

struct ValidateState<'state, 'item> {
    inner: &'state State<'item>,
    context: ValidateContext<'item>,
    const_mark_span: Option<Span>,
    param_constness: ParamConstness,
}

impl<'state, 'item> ValidateState<'state, 'item> {
    fn new(state: &'state State<'item>, files: &'item [ReadFile], root_path: &'item Path) -> Self {
        Self {
            inner: state,
            context: ValidateContext::new(files, root_path),
            const_mark_span: None,
            param_constness: ParamConstness::ExplicitOnly,
        }
    }

    fn span_location(&self, span: Span) -> LogLocation {
        self.context.location(span)
    }

    fn add_log(&mut self, log: Log) {
        self.context.logs.push(log);
    }

    fn with_param_constness<O>(
        &mut self,
        param_constness: ParamConstness,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let previous_param_constness = self.param_constness;
        self.param_constness = param_constness;
        let output = callback(self);
        self.param_constness = previous_param_constness;
        output
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

#[derive(Debug, Clone, Copy)]
enum ParamConstness {
    ExplicitOnly,
    All,
}

pub(crate) fn validate_modules<'item>(
    root_path: &'item Path,
    files: &'item [ReadFile],
    modules: &'item [Module],
    is_warning_treated_as_error: bool,
    state: &State<'item>,
) -> Result<Vec<Log>, Vec<Log>> {
    let mut state = ValidateState::new(state, files, root_path);
    let mut is_error_detected = false;
    for module in modules {
        is_error_detected |= validate_module(module, &mut state).is_err();
    }
    if !is_error_detected {
        // Import usage is checked in dedicated pass to avoid false positive
        // in case of cyclic dependencies.
        for module in modules {
            validate_module_import_usage(module, &mut state);
        }
    }
    state.context.logs.sort_by_key(Log::sort_key);
    let is_log_error = state
        .context
        .logs
        .iter()
        .any(|log| is_log_error(log, is_warning_treated_as_error));
    let logs = mem::take(&mut state.context.logs);
    if is_log_error { Err(logs) } else { Ok(logs) }
}

fn is_log_error(log: &Log, is_warning_treated_as_error: bool) -> bool {
    log.level == LogLevel::Error || (is_warning_treated_as_error && log.level == LogLevel::Warning)
}

fn validate_module<'item>(
    module: &'item Module,
    state: &mut ValidateState<'_, 'item>,
) -> Result<(), ValidateError> {
    let mut is_module_valid = true;
    let mut are_imports_finished = false;
    for item in &module.items {
        if let Item::Import(import) = item {
            is_module_valid &= validate_import(import, !are_imports_finished, state).is_ok();
        } else {
            are_imports_finished = true;
        }
    }
    if !is_module_valid {
        return Err(ValidateError);
    }
    for item in &module.items {
        is_module_valid &= items::validate_item(item, state).is_ok();
    }
    if !is_module_valid {
        return Err(ValidateError);
    }
    Ok(())
}

fn validate_import(
    import: &Import,
    is_top_import: bool,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    let is_found = import.imported_file_index.is_some();
    validators::import::check_found(is_found, &import.segments, state)?;
    validators::import::check_top(is_top_import, import.span, state)?;
    validators::import::check_self_import(import.imported_file_index, import.span, state);
    for &segment in &import.segments {
        if let ImportSegment::Name(span) = segment {
            validators::ident::check_case(span, naming::IMPORT_ALLOWED_CASES, state);
        }
    }
    Ok(())
}

fn validate_module_import_usage(module: &Module, state: &mut ValidateState<'_, '_>) {
    for item in &module.items {
        if let Item::Import(import) = item {
            validators::import::check_usage(
                import.id,
                import.imported_file_index,
                import.span,
                import.pub_keyword_span.is_some(),
                &import.segments,
                state,
            );
        }
    }
}
