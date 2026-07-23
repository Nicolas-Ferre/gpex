use crate::compiler::indexing::item_ref::ItemRef;
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

pub(crate) fn scan_var(node: &VarDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    state.dependencies = Dependencies::new();
    scan_var_inner(node, state)
}

pub(crate) fn scan_const(node: &ConstDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    state.dependencies = Dependencies::new();
    scan_const_inner(node, state)
}

pub(crate) fn scan_fn(node: &FnDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    state.dependencies = Dependencies::new();
    scan_fn_inner(node, state)
}

fn scan_call(node: &Call, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    for arg in &node.args {
        scan_expr(&arg.value, state)?;
    }
    if let Some(&source) = state.sources.get(&node.id) {
        scan_item(source, node.span, state)
    } else {
        // Covers case where there is function circular dependency from their signature.
        // As call source resolution is not done in this case, candidates are followed instead.
        let candidates = state
            .candidate_sources
            .get(&node.id)
            .cloned() // TODO: avoid costly Vec clone
            .unwrap_or_default();
        for source in candidates {
            scan_item(source, node.span, state)?;
        }
        Ok(())
    }
}

fn scan_item<'item>(
    node: ItemRef<'item>,
    ref_span: Span,
    state: &mut State<'item>,
) -> Result<(), Vec<Span>> {
    state.dependencies.enter_item(ref_span, node)?;
    match node {
        ItemRef::Var(child) => scan_var_inner(child, state)?,
        ItemRef::Const(child) => scan_const_inner(child, state)?,
        ItemRef::Fn(child) => scan_fn_inner(child, state)?,
        ItemRef::Struct(_) | ItemRef::Param(_) => (),
    }
    state.dependencies.exit_item();
    Ok(())
}

fn scan_var_inner(node: &VarDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_expr(&node.default_value, state)
}

fn scan_const_inner(node: &ConstDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_expr(&node.value, state)
}

fn scan_fn_inner(node: &FnDefinition, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_params(&node.params, state)?;
    if let Some(return_type) = &node.return_type {
        scan_expr(return_type, state)?;
    }
    if let FnBody::Statements(body) = &node.body {
        for statement in &body.statements {
            scan_statement(statement, state)?;
        }
    }
    Ok(())
}

fn scan_statement(node: &Statement, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    match node {
        Statement::Return(child) => scan_expr(&child.value, state)?,
        Statement::Assignment(child) => {
            scan_expr(&child.assigned, state)?;
            scan_expr(&child.value, state)?;
        }
    }
    Ok(())
}

fn scan_params(node: &ParamGroup, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    for param in &node.params {
        scan_param(param, state)?;
    }
    Ok(())
}

fn scan_param(node: &Param, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    scan_expr(&node.type_, state)?;
    if let Some(requirement) = &node.requirement {
        scan_expr(&requirement.condition, state)?;
    }
    Ok(())
}

fn scan_expr(node: &Expr, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    match node {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_) => Ok(()),
        Expr::Call(node) => scan_call(node, state), // no-fn-check (recursivity)
        Expr::Ident(node) => scan_ident(node, state),
    }
}

fn scan_ident(node: &Ident, state: &mut State<'_>) -> Result<(), Vec<Span>> {
    if let Some(&source) = state.sources.get(&node.id) {
        scan_item(source, node.span, state) // no-fn-check (recursivity)
    } else {
        Ok(())
    }
}
