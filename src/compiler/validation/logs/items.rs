use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogInner, LogLevel};

pub(crate) fn circular_dependencies(
    item_name: &str,
    item_name_span: Span,
    dependency_spans: &[Span],
    state: &ValidateState<'_, '_>,
) -> Log {
    debug_assert!(!dependency_spans.is_empty());
    Log {
        level: LogLevel::Error,
        msg: format!("`{item_name}` item has circular dependencies"),
        location: Some(state.span_location(item_name_span)),
        inner: dependency_spans
            .iter()
            .enumerate()
            .map(|(index, dependency_span)| LogInner {
                level: LogLevel::Info,
                msg: if index == dependency_spans.len() - 1 {
                    "depends on itself".into()
                } else {
                    "depends on this item".into()
                },
                location: Some(state.span_location(*dependency_span)),
            })
            .collect(),
    }
}

pub(crate) fn duplicate_definition(
    item_key: &str,
    item_name_span: Span,
    previous_item_name_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{item_key}` item defined multiple times"),
        location: Some(state.span_location(item_name_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "item also defined here".into(),
            location: Some(state.span_location(previous_item_name_span)),
        }],
    }
}

pub(crate) fn duplicate_param(
    param_name: &str,
    param_name_span: Span,
    previous_param_name_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{param_name}` parameter defined multiple times"),
        location: Some(state.span_location(param_name_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "parameter also defined here".into(),
            location: Some(state.span_location(previous_param_name_span)),
        }],
    }
}

pub(crate) fn forbidden_intrinsic(
    intrinsic_keyword_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "forbidden `intrinsic` item outside prelude".into(),
        location: Some(state.span_location(intrinsic_keyword_span)),
        inner: vec![],
    }
}

pub(crate) fn not_found_with_similar_candidates(
    item_key: &str,
    item_ref_span: Span,
    candidate_spans: impl IntoIterator<Item = Span>,
    state: &ValidateState<'_, '_>,
) -> Log {
    not_found(
        item_key,
        item_ref_span,
        candidate_spans
            .into_iter()
            .map(|candidate_span| LogInner {
                level: LogLevel::Info,
                msg: "similar candidate".into(),
                location: Some(state.span_location(candidate_span)),
            })
            .collect(),
        state,
    )
}

pub(crate) fn not_found_with_priv_candidate(
    item_key: &str,
    item_ref_span: Span,
    candidate_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    not_found(
        item_key,
        item_ref_span,
        vec![LogInner {
            level: LogLevel::Info,
            msg: "item not qualified with `pub`".into(),
            location: Some(state.span_location(candidate_span)),
        }],
        state,
    )
}

pub(crate) fn not_found_with_importable_candidate<'path>(
    item_key: &str,
    item_ref_span: Span,
    candidate_dot_paths_and_spans: impl IntoIterator<Item = (&'path str, Span)>,
    state: &ValidateState<'_, '_>,
) -> Log {
    not_found(
        item_key,
        item_ref_span,
        candidate_dot_paths_and_spans
            .into_iter()
            .map(|(dot_path, importable_item_name_span)| LogInner {
                level: LogLevel::Info,
                msg: format!("item can be imported from `{dot_path}`"),
                location: Some(state.span_location(importable_item_name_span)),
            })
            .collect(),
        state,
    )
}

pub(crate) fn duplicate_fn(
    fn_key: &str,
    fn_signature_span: Span,
    previous_fn_signature_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: format!("`{fn_key}` function defined multiple times"),
        location: Some(state.span_location(fn_signature_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "function also defined here".into(),
            location: Some(state.span_location(previous_fn_signature_span)),
        }],
    }
}

pub(crate) fn unused(
    item_key: &str,
    replacement: Option<&str>,
    item_name_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: format!("`{item_key}` item unused"),
        location: Some(state.span_location(item_name_span)),
        inner: replacement.map(super::replacement).into_iter().collect(),
    }
}

pub(crate) fn pub_with_ignored_name(
    item_key: &str,
    item_name: &str,
    item_name_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    let replacement = item_name
        .strip_prefix('_')
        .filter(|replacement| !replacement.is_empty());
    Log {
        level: LogLevel::Warning,
        msg: format!("`{item_key}` item public but name starting with `_`"),
        location: Some(state.span_location(item_name_span)),
        inner: replacement.map(super::replacement).into_iter().collect(),
    }
}

pub(crate) fn used_with_ignored_name(
    item_key: &str,
    item_name: &str,
    item_name_span: Span,
    item_ref_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: format!("`{item_key}` item used but name starting with `_`"),
        location: Some(state.span_location(item_name_span)),
        inner: vec![
            LogInner {
                level: LogLevel::Info,
                msg: "item used here".into(),
                location: Some(state.span_location(item_ref_span)),
            },
            super::replacement(item_name.strip_prefix('_').unwrap_or(item_name)),
        ],
    }
}

fn not_found(
    item_key: &str,
    item_ref_span: Span,
    inner_logs: Vec<LogInner>,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{item_key}` item not found"),
        location: Some(state.span_location(item_ref_span)),
        inner: inner_logs,
    }
}
