use crate::compiler::indexing::type_narrowing::LogicalTypeNarrowing;
use crate::compiler::indexing::{IndexState, exprs, type_narrowing};
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::items::params::ParamRequirement;
use crate::compiler::parsing::statements::Statement;
use std::rc::Rc;

pub(super) fn index_fn_const_parts<'item>(
    fn_: &'item FnDefinition,
    state: &mut IndexState<'_, 'item>,
) {
    state.type_fact_context = Rc::default();
    index_fn_params(fn_, state);
    index_fn_return_type(fn_, state);
    if fn_.const_keyword_span.is_some() {
        index_fn_body(fn_, state);
    }
}

pub(super) fn index_fn_not_const_parts<'item>(
    fn_: &'item FnDefinition,
    state: &mut IndexState<'_, 'item>,
) {
    if fn_.const_keyword_span.is_none() {
        state.type_fact_context = Rc::default();
        add_fn_requirement_type_facts(fn_, state);
        index_fn_body(fn_, state);
    }
}

fn index_fn_params<'item>(fn_: &'item FnDefinition, state: &mut IndexState<'_, 'item>) {
    for param in &fn_.params.params {
        exprs::index_expr(&param.type_, state);
        if let Some(requirement) = &param.requirement {
            exprs::index_expr(&requirement.condition, state);
            add_param_requirement_type_facts(requirement, state);
        }
    }
}

fn add_fn_requirement_type_facts<'item>(
    fn_: &'item FnDefinition,
    state: &mut IndexState<'_, 'item>,
) {
    for param in &fn_.params.params {
        if let Some(requirement) = &param.requirement {
            add_param_requirement_type_facts(requirement, state);
        }
    }
}

fn add_param_requirement_type_facts<'item>(
    requirement: &'item ParamRequirement,
    state: &mut IndexState<'_, 'item>,
) {
    type_narrowing::add_expr_type_facts(&requirement.condition, LogicalTypeNarrowing::And, state);
}

fn index_fn_return_type<'item>(fn_: &'item FnDefinition, state: &mut IndexState<'_, 'item>) {
    if let Some(return_type) = &fn_.return_type {
        exprs::index_expr(return_type, state);
    }
}

fn index_fn_body<'item>(fn_: &'item FnDefinition, state: &mut IndexState<'_, 'item>) {
    if let FnBody::Statements(body) = &fn_.body {
        for statement in &body.statements {
            index_statement_refs(statement, state);
        }
    }
}

fn index_statement_refs<'item>(statement: &'item Statement, state: &mut IndexState<'_, 'item>) {
    match statement {
        Statement::Return(return_) => exprs::index_expr(&return_.value, state),
        Statement::Assignment(assignment) => {
            exprs::index_expr(&assignment.assigned, state);
            exprs::index_expr(&assignment.value, state);
        }
    }
}
