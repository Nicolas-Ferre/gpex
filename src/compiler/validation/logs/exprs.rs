use crate::compiler::validation::ValidateState;
use crate::utils::parsing::span::Span;
use crate::{Log, LogInner, LogLevel};

pub(crate) fn invalid_type(
    actual_type: &str,
    actual_span: Span,
    expected_type: &str,
    expected_span: Option<Span>,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("expression with invalid type `{actual_type}`"),
        location: Some(state.span_location(actual_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: format!("expected `{expected_type}` type"),
            location: expected_span.map(|span| state.span_location(span)),
        }],
    }
}

pub(crate) fn non_const(
    expr_span: Span,
    const_mark_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "expression not constant".into(),
        location: Some(state.span_location(expr_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "expression must be constant".into(),
            location: Some(state.span_location(const_mark_span)),
        }],
    }
}

pub(crate) fn f32_const_out_of_bounds(expr_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "`f32` constant expression out of bounds".into(),
        location: Some(state.span_location(expr_span)),
        inner: vec![],
    }
}

pub(crate) fn not_ref(expr_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "expression is not a reference".into(),
        location: Some(state.span_location(expr_span)),
        inner: vec![],
    }
}

pub(crate) fn invalid_wildcard(expr_span: Span, state: &ValidateState<'_, '_>) -> Log {
    Log {
        level: LogLevel::Error,
        msg: "invalid wildcard expression".into(),
        location: Some(state.span_location(expr_span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "wildcards are only allowed as function parameter types".into(),
            location: None,
        }],
    }
}

pub(crate) fn literal_out_of_bounds(
    type_name: &str,
    literal_span: Span,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Error,
        msg: format!("`{type_name}` literal out of bounds"),
        location: Some(state.span_location(literal_span)),
        inner: vec![],
    }
}

pub(crate) fn mul_add_candidate(
    expr_span: Span,
    replacement: &str,
    state: &ValidateState<'_, '_>,
) -> Log {
    Log {
        level: LogLevel::Warning,
        msg: "candidate expression for `mul_add()`".into(),
        location: Some(state.span_location(expr_span)),
        inner: vec![super::replacement(replacement)],
    }
}
