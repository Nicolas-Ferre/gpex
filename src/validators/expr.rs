use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::span::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_types(
    actual_span: Span,
    actual_type: Type<'_>,
    expected_span: Option<Span>,
    expected_type: Type<'_>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if let Type::Struct(expected_type) = expected_type
        && let Type::Struct(actual_type) = actual_type
    {
        if actual_type == expected_type {
            Ok(())
        } else {
            context.logs.push(Log {
                level: LogLevel::Error,
                msg: format!("expression with invalid type `{}`", actual_type.name),
                location: Some(context.location(actual_span)),
                inner: vec![LogInner {
                    level: LogLevel::Info,
                    msg: format!("expected `{}` type", expected_type.name),
                    location: expected_span.map(|span| context.location(span)),
                }],
            });
            Err(ValidateError)
        }
    } else {
        Err(ValidateError)
    }
}

pub(crate) fn check_const_value(
    source: ItemRef<'_>,
    span: Span,
    const_mark_span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    // This validator function is always called from a `const` context,
    // so we cannot be inside a non-`const` function.
    let is_in_const_fn = true;
    if source.is_const(is_in_const_fn) {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "expression not constant".into(),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "expression must be constant".into(),
                location: Some(context.location(const_mark_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_no_return_type(
    node: impl NodeRef,
    span: Span,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if let Some(&ItemRef::Fn(fn_)) = indexes.sources.get(&node.id())
        && fn_.return_type.is_none()
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!(
                "called function `{}` with no return type",
                fn_.displayed_key(indexes)
            ),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "function has no return type".into(),
                location: Some(context.location(fn_.signature_span)),
            }],
        });
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_has_return_type(
    node: impl NodeRef,
    span: Span,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if let Some(&ItemRef::Fn(fn_)) = indexes.sources.get(&node.id())
        && fn_.return_type.is_some()
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!(
                "repeated function `{}` with a return type",
                fn_.displayed_key(indexes)
            ),
            location: Some(context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "function has a return type".into(),
                location: Some(context.location(fn_.signature_span)),
            }],
        });
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_ref(expr: &Expr, context: &mut ValidateContext<'_>, indexes: &Indexes<'_>) {
    if expr.is_ref(indexes) == Some(false) {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "expression is not a reference".into(),
            location: Some(context.location(expr.span())),
            inner: vec![],
        });
    }
}
