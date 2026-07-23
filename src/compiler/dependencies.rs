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

pub(crate) fn scan_var(var: &VarDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    state.dependencies = Dependencies::new();
    scan_var_inner(var, state)
}

pub(crate) fn scan_const(const_: &ConstDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    state.dependencies = Dependencies::new();
    scan_const_inner(const_, state)
}

pub(crate) fn scan_fn(fn_: &FnDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    state.dependencies = Dependencies::new();
    scan_fn_inner(fn_, state)
}

fn scan_call(call: &Call, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    for arg in &call.args {
        scan_expr(&arg.value, state)?;
    }
    if let Some(&source) = state.sources.get(&call.id) {
        scan_item(source, call.span, state)
    } else {
        // Covers case where there is function circular dependency from their signature.
        // As call source resolution is not done in this case, candidates are followed instead.
        let candidate_count = state.candidate_sources.get(&call.id).map_or(0, Vec::len);
        for index in 0..candidate_count {
            let source = state.candidate_sources[&call.id][index];
            scan_item(source, call.span, state)?;
        }
        Ok(())
    }
}

fn scan_item<'item>(
    item: ItemRef<'item>,
    ref_span: Span,
    state: &mut State<'item>,
) -> Result<(), Vec<Span>> {
    state.dependencies.enter_item(ref_span, item)?;
    match item {
        ItemRef::Var(child) => scan_var_inner(child, state)?,
        ItemRef::Const(child) => scan_const_inner(child, state)?,
        ItemRef::Fn(child) => scan_fn_inner(child, state)?,
        ItemRef::Struct(_) | ItemRef::Param(_) => (),
    }
    state.dependencies.exit_item();
    Ok(())
}

fn scan_var_inner(var: &VarDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_expr(&var.default_value, state)
}

fn scan_const_inner(const_: &ConstDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_expr(&const_.value, state)
}

fn scan_fn_inner(fn_: &FnDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_params(&fn_.params, state)?;
    if let Some(return_type) = &fn_.return_type {
        scan_expr(return_type, state)?;
    }
    if let FnBody::Statements(body) = &fn_.body {
        for statement in &body.statements {
            scan_statement(statement, state)?;
        }
    }
    Ok(())
}

fn scan_statement(statement: &Statement, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    match statement {
        Statement::Return(child) => scan_expr(&child.value, state)?,
        Statement::Assignment(child) => {
            scan_expr(&child.assigned, state)?;
            scan_expr(&child.value, state)?;
        }
    }
    Ok(())
}

fn scan_params(params: &ParamGroup, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    for param in &params.params {
        scan_param(param, state)?;
    }
    Ok(())
}

fn scan_param(param: &Param, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_expr(&param.type_, state)?;
    if let Some(requirement) = &param.requirement {
        scan_expr(&requirement.condition, state)?;
    }
    Ok(())
}

fn scan_expr(expr: &Expr, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    match expr {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_) => Ok(()),
        Expr::Call(call) => scan_call(call, state), // no-fn-check (recursivity)
        Expr::Ident(ident) => scan_ident(ident, state),
    }
}

fn scan_ident(ident: &Ident, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    if let Some(&source) = state.sources.get(&ident.id) {
        scan_item(source, ident.span, state) // no-fn-check (recursivity)
    } else {
        Ok(())
    }
}
