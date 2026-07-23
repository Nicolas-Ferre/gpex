use crate::compiler::item_ref::ItemRef;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{BinaryCompilerImplFn, CompilerImplFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::refs;
use crate::compiler::state::ParamConstness;
use crate::compiler::state::State;
use crate::compiler::values::types::Type;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_types(
    actual_span: Span,
    actual_type: Type<'_>,
    expected_span: Option<Span>,
    expected_type: Type<'_>,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    if !actual_type.is_comparable() || !expected_type.is_comparable() {
        Err(ValidateError)
    } else if actual_type == expected_type {
        Ok(())
    } else {
        state.add_log(Log {
            level: LogLevel::Error,
            msg: format!("expression with invalid type `{}`", actual_type.name()?),
            location: Some(state.span_location(actual_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: format!("expected `{}` type", expected_type.name()?),
                location: expected_span.map(|span| state.span_location(span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_const_value(
    source: ItemRef<'_>,
    span: Span,
    const_mark_span: Span,
    param_constness: ParamConstness,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    if is_item_const(source, param_constness) {
        Ok(())
    } else {
        state.add_log(Log {
            level: LogLevel::Error,
            msg: "expression not constant".into(),
            location: Some(state.span_location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "expression must be constant".into(),
                location: Some(state.span_location(const_mark_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_f32_const_bounds(
    is_out_of_bounds: bool,
    span: Span,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    if is_out_of_bounds {
        state.add_log(Log {
            level: LogLevel::Error,
            msg: "`f32` constant expression out of bounds".into(),
            location: Some(state.span_location(span)),
            inner: vec![],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_mul_add_candidate(
    source: ItemRef<'_>,
    call: &Call,
    are_all_args_f32: bool,
    state: &mut State<'_>,
) {
    let ItemRef::Fn(source) = source else {
        unreachable!("calls can only be functions")
    };
    if !are_all_args_f32
        || source.compilerimpl() != Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Add))
        || !call
            .args
            .iter()
            .any(|arg| is_expr_compilerimpl_mul(&arg.value, state))
    {
        return;
    }
    state.add_log(Log {
        level: LogLevel::Warning,
        msg: "candidate expression for `mul_add()`".into(),
        location: Some(state.span_location(call.span)),
        inner: vec![],
    });
}

pub(crate) fn check_arg_name(
    arg: &Arg,
    param: &Param,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    let Some(name) = &arg.name else {
        return Ok(());
    };
    if name == &param.name {
        Ok(())
    } else {
        state.add_log(Log {
            level: LogLevel::Error,
            msg: format!("`{name}` argument name not matching parameter"),
            location: arg.name_span.map(|span| state.span_location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: format!("expected `{}` parameter name", param.name),
                location: Some(state.span_location(param.name_span)),
            }],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_no_return_type(
    node: impl NodeRef,
    span: Span,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    if let Some(ItemRef::Fn(fn_)) = state.sources.get(&node.id()).copied()
        && fn_.return_type.is_none()
    {
        let fn_key = key_rendering::fn_key(fn_, state)?;
        state.add_log(Log {
            level: LogLevel::Error,
            msg: format!("called function `{fn_key}` with no return type"),
            location: Some(state.span_location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "function has no return type".into(),
                location: Some(state.span_location(fn_.signature_span_with_return)),
            }],
        });
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_has_return_type(
    node: impl NodeRef,
    span: Span,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    if let Some(ItemRef::Fn(fn_)) = state.sources.get(&node.id()).copied()
        && fn_.return_type.is_some()
    {
        let fn_key = key_rendering::fn_key(fn_, state)?;
        state.add_log(Log {
            level: LogLevel::Error,
            msg: format!("repeated function `{fn_key}` with a return type"),
            location: Some(state.span_location(span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "function has a return type".into(),
                location: Some(state.span_location(fn_.signature_span_with_return)),
            }],
        });
        return Err(ValidateError);
    }
    Ok(())
}

pub(crate) fn check_ref(expr: &Expr, state: &mut State<'_>) {
    if refs::is_expr_ref(expr, state) == Some(false) {
        state.add_log(Log {
            level: LogLevel::Error,
            msg: "expression is not a reference".into(),
            location: Some(state.span_location(expr.span())),
            inner: vec![],
        });
    }
}

pub(crate) fn report_invalid_wildcard_location(
    span: Span,
    state: &mut State<'_>,
) -> Result<(), ValidateError> {
    state.add_log(Log {
        level: LogLevel::Error,
        msg: "invalid wildcard expression".into(),
        location: Some(state.span_location(span)),
        inner: vec![LogInner {
            level: LogLevel::Info,
            msg: "wildcards are only allowed as function parameter types".into(),
            location: None,
        }],
    });
    Err(ValidateError)
}

fn is_item_const(item: ItemRef<'_>, param_constness: ParamConstness) -> bool {
    match item {
        ItemRef::Var(_) => false,
        ItemRef::Const(_) | ItemRef::Struct(_) => true,
        ItemRef::Fn(fn_) => fn_.const_keyword_span.is_some(),
        ItemRef::Param(param) => match param_constness {
            ParamConstness::ExplicitOnly => param.const_mark_span().is_some(),
            ParamConstness::All => true,
        },
    }
}

fn is_expr_compilerimpl_mul(expr: &Expr, state: &State<'_>) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source))
            if source.compilerimpl()
                == Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Mul))
    )
}
