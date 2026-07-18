use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{BinaryCompilerImplFn, CompilerImplFn};
use crate::compiler::parsing::items::params::Param;
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

pub(crate) fn check_f32_const_bounds(
    is_out_of_bounds: bool,
    span: Span,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if is_out_of_bounds {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "`f32` constant expression out of bounds".into(),
            location: Some(context.location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_mul_add_candidate(
    source: ItemRef<'_>,
    first_arg_type: Type<'_>,
    node: &Call,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    let ItemRef::Fn(source) = source else {
        return;
    };
    if source.compilerimpl() != Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Add))
        || first_arg_type != Type::Struct(indexes.search_prelude_type("f32"))
        || !node
            .args
            .iter()
            .any(|arg| is_expr_compilerimpl_mul(&arg.value, indexes))
    {
        return;
    }
    context.logs.push(Log {
        level: LogLevel::Warning,
        msg: "candidate expression for `mul_add()`".into(),
        location: Some(context.location(node.span)),
        inner: vec![],
    });
}

pub(crate) fn check_arg_name(
    arg: &Arg,
    param: &Param,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let Some(name) = &arg.name else {
        return Ok(());
    };
    if name == &param.name {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{name}` argument name not matching parameter"),
            location: arg.name_span.map(|span| context.location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: format!("expected `{}` parameter name", param.name),
                location: Some(context.location(param.name_span)),
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
                location: Some(context.location(fn_.signature_span_with_return)),
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
                location: Some(context.location(fn_.signature_span_with_return)),
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

fn is_expr_compilerimpl_mul(node: &Expr, indexes: &Indexes<'_>) -> bool {
    let Expr::Call(node) = node else {
        return false;
    };
    matches!(
        indexes.sources.get(&node.id),
        Some(ItemRef::Fn(source))
            if source.compilerimpl()
                == Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Mul))
    )
}
