use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::refs::RefChecker;
use crate::compiler::validation::ParamConstness;
use crate::compiler::values::types::Type;
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
    if !actual_type.is_comparable() || !expected_type.is_comparable() {
        Err(ValidateError)
    } else if actual_type == expected_type {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("expression with invalid type `{}`", actual_type.name()?),
            location: Some(context.location(actual_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: format!("expected `{}` type", expected_type.name()?),
                location: expected_span.map(|span| context.location(span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_const_value(
    source: ItemRef<'_>,
    span: Span,
    const_mark_span: Span,
    context: &mut ValidateContext<'_>,
    param_constness: ParamConstness,
) -> Result<(), ValidateError> {
    if is_item_const(source, param_constness) {
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
            msg: format!("called function `{fn_key}` with no return type"),
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
            msg: format!("repeated function `{fn_key}` with a return type"),
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

pub(crate) fn check_ref(node: &Expr, context: &mut ValidateContext<'_>, indexes: &Indexes<'_>) {
    if RefChecker::new(indexes).is_expr_ref(node) == Some(false) {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "expression is not a reference".into(),
            location: Some(context.location(node.span())),
            inner: vec![],
        });
    }
}

pub(crate) fn report_invalid_wildcard_location(
    span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    context.logs.push(Log {
        level: LogLevel::Error,
        msg: "invalid wildcard expression".into(),
        location: Some(context.location(span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "wildcards are only allowed as function parameter types".into(),
            location: None,
        }],
    });
    Err(ValidateError)
}

fn is_item_const(node: ItemRef<'_>, param_constness: ParamConstness) -> bool {
    match node {
        ItemRef::Var(_) => false,
        ItemRef::Const(_) | ItemRef::Struct(_) => true,
        ItemRef::Fn(node) => node.const_keyword_span.is_some(),
        ItemRef::Param(node) => match param_constness {
            ParamConstness::ExplicitOnly => node.const_mark_span().is_some(),
            ParamConstness::All => true,
        },
    }
}
