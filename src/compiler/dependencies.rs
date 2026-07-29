use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::parsing::statements::Statement;
use crate::compiler::state::State;
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::span::Span;

pub(crate) fn scan_var<'item>(
    var: &VarDefinition,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_var_inner(var, dependencies, state)
}

pub(crate) fn scan_const<'item>(
    const_: &ConstDefinition,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_const_inner(const_, dependencies, state)
}

pub(crate) fn scan_fn<'item>(
    fn_: &FnDefinition,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_fn_inner(fn_, dependencies, state)
}

fn scan_call<'item>(
    call: &Call,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    for arg in &call.args {
        scan_expr(&arg.value, dependencies, state)?;
    }
    if let Some(&source) = state.sources.get(&call.id) {
        scan_item(source, call.span, dependencies, state)
    } else {
        // Covers case where there is function circular dependency from their signature.
        // As call source resolution is not done in this case, candidates are followed instead.
        if let Some(candidates) = state.candidate_sources.get(&call.id) {
            for &source in candidates {
                scan_item(source, call.span, dependencies, state)?;
            }
        }
        Ok(())
    }
}

fn scan_item<'item>(
    item: ItemRef<'item>,
    ref_span: Span,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    dependencies.enter_item(ref_span, item)?;
    match item {
        ItemRef::Var(child) => scan_var_inner(child, dependencies, state)?,
        ItemRef::Const(child) => scan_const_inner(child, dependencies, state)?,
        ItemRef::Fn(child) => scan_fn_inner(child, dependencies, state)?,
        ItemRef::Struct(_) | ItemRef::Param(_) => (),
    }
    dependencies.exit_item();
    Ok(())
}

fn scan_var_inner<'item>(
    var: &VarDefinition,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_expr(&var.default_value, dependencies, state)
}

fn scan_const_inner<'item>(
    const_: &ConstDefinition,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_expr(&const_.value, dependencies, state)
}

fn scan_fn_inner<'item>(
    fn_: &FnDefinition,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_params(&fn_.params, dependencies, state)?;
    if let Some(return_type) = &fn_.return_type {
        scan_expr(return_type, dependencies, state)?;
    }
    if let FnBody::Statements(body) = &fn_.body {
        for statement in &body.statements {
            scan_statement(statement, dependencies, state)?;
        }
    }
    Ok(())
}

fn scan_statement<'item>(
    statement: &Statement,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    match statement {
        Statement::Return(child) => scan_expr(&child.value, dependencies, state)?,
        Statement::Assignment(child) => {
            scan_expr(&child.assigned, dependencies, state)?;
            scan_expr(&child.value, dependencies, state)?;
        }
    }
    Ok(())
}

fn scan_params<'item>(
    params: &ParamGroup,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    for param in &params.params {
        scan_param(param, dependencies, state)?;
    }
    Ok(())
}

fn scan_param<'item>(
    param: &Param,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    scan_expr(&param.type_, dependencies, state)?;
    if let Some(requirement) = &param.requirement {
        scan_expr(&requirement.condition, dependencies, state)?;
    }
    Ok(())
}

fn scan_expr<'item>(
    expr: &Expr,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    match expr {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_) => Ok(()),
        Expr::Call(call) => scan_call(call, dependencies, state), // no-fn-check (recursivity)
        Expr::Ident(ident) => scan_ident(ident, dependencies, state),
        Expr::Parenthesized(parenthesized) => scan_expr(&parenthesized.value, dependencies, state),
    }
}

fn scan_ident<'item>(
    ident: &Ident,
    dependencies: &mut Dependencies<ItemRef<'item>>,
    state: &State<'item>,
) -> Result<(), Vec<Span>> {
    if let Some(&source) = state.sources.get(&ident.id) {
        scan_item(source, ident.span, dependencies, state) // no-fn-check (recursivity)
    } else {
        Ok(())
    }
}
