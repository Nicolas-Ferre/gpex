use crate::compiler::parsing::items::imports::{Import, ImportSegment};
use crate::compiler::validation::{ValidateState, logs, naming};
use crate::utils::parsing::span::SpanProps;
use crate::utils::validation::ValidateError;
use itertools::Itertools;

pub(super) fn validate_import(
    import: &Import,
    is_import_at_top: bool,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    debug_assert!(!import.segments.is_empty());
    if import.imported_file_index.is_none() {
        let dot_path = dot_path_from_segments(&import.segments, state);
        let context = &state.context;
        let fs_path = ImportSegment::fs_path(&import.segments, context, context.root_path);
        let first_segment = import.segments[0];
        let last_segment = import.segments[import.segments.len() - 1];
        let segments_span = first_segment.span().until(last_segment.span());
        state.add_log(logs::imports::not_found(
            &dot_path,
            segments_span,
            &fs_path,
            state,
        ));
        return Err(ValidateError);
    }
    if !is_import_at_top {
        state.add_log(logs::imports::not_at_top(import.span, state));
        return Err(ValidateError);
    }
    if import.imported_file_index == Some(import.span.file_index) {
        state.add_log(logs::imports::self_import(import.span, state));
    }
    validate_segments(import, state);
    Ok(())
}

pub(super) fn validate_import_usage(import: &Import, state: &mut ValidateState<'_, '_>) {
    let is_used = state
        .inner
        .imports
        .is_used(import.span.file_index, import.id);
    let is_self_import = import.imported_file_index == Some(import.span.file_index);
    let is_pub = import.pub_keyword_span.is_some();
    if !is_self_import && !is_pub && !is_used {
        let dot_path = dot_path_from_segments(&import.segments, state);
        state.add_log(logs::imports::unused(&dot_path, import.span, state));
    }
}

fn validate_segments(import: &Import, state: &mut ValidateState<'_, '_>) {
    for &segment in &import.segments {
        if let ImportSegment::Name(span) = segment {
            naming::validate_case(span, false, naming::IMPORT_ALLOWED_CASES, state);
        }
    }
}

fn dot_path_from_segments(segments: &[ImportSegment], state: &ValidateState<'_, '_>) -> String {
    segments
        .iter()
        .map(|&segment| state.context.slice(segment.span()))
        .join(".")
}
