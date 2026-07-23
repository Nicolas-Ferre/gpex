mod exprs;
mod fns;
mod items;
mod naming;
mod validators;

use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::imports::{Import, ImportSegment};
use crate::compiler::parsing::modules::Module;
use crate::compiler::state::State;
use crate::utils::validation::ValidateError;
use crate::{Log, LogLevel};
use std::mem;

pub(crate) fn validate_modules<'item>(
    modules: &'item [Module],
    is_warning_treated_as_error: bool,
    state: &mut State<'item>,
) -> Result<Vec<Log>, Vec<Log>> {
    let mut is_error_detected = false;
    for module in modules {
        is_error_detected |= validate_module(module, state).is_err();
    }
    if !is_error_detected {
        // Import usage is checked in dedicated pass to avoid false positive
        // in case of cyclic dependencies.
        for module in modules {
            validate_module_import_usage(module, state);
        }
    }
    state.validation_context.logs.sort_by_key(Log::sort_key);
    let is_log_error = state
        .validation_context
        .logs
        .iter()
        .any(|log| is_log_error(log, is_warning_treated_as_error));
    let logs = mem::take(&mut state.validation_context.logs);
    if is_log_error { Err(logs) } else { Ok(logs) }
}

fn is_log_error(log: &Log, is_warning_treated_as_error: bool) -> bool {
    log.level == LogLevel::Error || (is_warning_treated_as_error && log.level == LogLevel::Warning)
}

fn validate_module<'item>(
    module: &'item Module,
    state: &mut State<'item>,
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
    state: &mut State<'_>,
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

fn validate_module_import_usage(module: &Module, state: &mut State<'_>) {
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
