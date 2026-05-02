use crate::compiler::consts::ConstChecker;
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::refs::RefChecker;
use crate::compiler::types::Type;
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
    const_checker: &ConstChecker<'_, '_>,
) -> Result<(), ValidateError> {
    if const_checker.is_item_const(source) {
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
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("called function `{fn_key}` with no return type",),
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
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("repeated function `{fn_key}` with a return type",),
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
    if RefChecker::new(indexes).is_expr_ref(expr) == Some(false) {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "expression is not a reference".into(),
            location: Some(context.location(expr.span())),
            inner: vec![],
        });
    }
}
