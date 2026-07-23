use crate::compiler::parsing::items::imports::ImportSegment;
use crate::compiler::validation::{ValidateState, logs};
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::ValidateError;
use itertools::Itertools;

pub(crate) fn check_found(
    is_found: bool,
    segments: &[ImportSegment],
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    debug_assert!(!segments.is_empty());
    if is_found {
        Ok(())
    } else {
        let dot_path = dot_path_from_segments(segments, state);
        let context = &state.context;
        let fs_path = ImportSegment::fs_path(segments, context, context.root_path);
        let first_segment = segments[0];
        let last_segment = segments[segments.len() - 1];
        let segments_span = first_segment.span().until(last_segment.span());
        state.add_log(logs::imports::not_found(
            &dot_path,
            segments_span,
            &fs_path,
            state,
        ));
        Err(ValidateError)
    }
}

pub(crate) fn check_top(
    is_top: bool,
    span: Span,
    state: &mut ValidateState<'_, '_>,
) -> Result<(), ValidateError> {
    if is_top {
        Ok(())
    } else {
        state.add_log(logs::imports::not_at_top(span, state));
        Err(ValidateError)
    }
}

pub(crate) fn check_self_import(
    imported_file_index: Option<usize>,
    span: Span,
    state: &mut ValidateState<'_, '_>,
) {
    if let Some(imported_file_index) = imported_file_index
        && imported_file_index == span.file_index
    {
        state.add_log(logs::imports::self_import(span, state));
    }
}

pub(crate) fn check_usage(
    import_id: u64,
    imported_file_index: Option<usize>,
    span: Span,
    is_pub: bool,
    segments: &[ImportSegment],
    state: &mut ValidateState<'_, '_>,
) {
    let is_used = state.inner.imports.is_used(span.file_index, import_id);
    let is_self_import = imported_file_index == Some(span.file_index);
    if !is_self_import && !is_pub && !is_used {
        let dot_path = dot_path_from_segments(segments, state);
        state.add_log(logs::imports::unused(&dot_path, span, state));
    }
}

fn dot_path_from_segments(segments: &[ImportSegment], state: &ValidateState<'_, '_>) -> String {
    segments
        .iter()
        .map(|&segment| state.context.slice(segment.span()))
        .join(".")
}
