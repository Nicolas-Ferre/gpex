use crate::compiler::consts::{self, ConstValue};
use crate::compiler::indexing::{IndexState, exprs};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::queries;
use crate::compiler::state::{State, TypeFacts};
use crate::compiler::types;
use crate::utils::parsing::span::Span;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub(super) struct TypeNarrowingState<'item> {
    type_facts: HashMap<u64, Rc<TypeFacts<'item>>>,
}

struct TypeFact<'item> {
    param: &'item Param,
    type_: &'item StructDefinition,
    condition_node_id: u64,
    subject_span: Span,
}

pub(super) fn index_and_args<'item>(call: &'item Call, state: &mut IndexState<'_, 'item>) {
    exprs::index_expr(&call.args[0].value, state);
    let mut facts = vec![];
    collect_type_facts(&call.args[0].value, state.inner, &mut facts);
    let previous_facts = state.type_narrowing.type_facts.clone();
    for fact in facts {
        add_type_fact(fact, state);
    }
    exprs::index_expr(&call.args[1].value, state);
    state.type_narrowing.type_facts = previous_facts;
}

pub(super) fn index_ident<'item>(
    ident: &Ident,
    source: ItemRef<'item>,
    state: &mut IndexState<'_, 'item>,
) {
    if let ItemRef::Param(param) = source
        && let Some(facts) = state.type_narrowing.type_facts.get(&param.id).cloned()
    {
        state.set_expr_type_facts(ident.id, facts);
    }
}

fn add_type_fact<'item>(fact: TypeFact<'item>, state: &mut IndexState<'_, 'item>) {
    let item_id = fact.param.id;
    let item_type = types::param_type(fact.param, state.inner);
    let mut facts = state
        .type_narrowing
        .type_facts
        .get(&item_id)
        .map(|facts| facts.as_ref().clone())
        .unwrap_or_default();
    let was_type_contradicted = facts.is_type_contradicted(item_type);
    facts.add_required_type(fact.type_);
    let is_type_contradicted = facts.is_type_contradicted(item_type);
    let is_type_newly_contradicted = !was_type_contradicted && is_type_contradicted;
    let contradicted_type_fact_subject_span =
        is_type_newly_contradicted.then_some(fact.subject_span);
    state.inner.set_contradicted_type_fact_subject_span(
        fact.condition_node_id,
        contradicted_type_fact_subject_span,
    );
    state
        .type_narrowing
        .type_facts
        .insert(item_id, Rc::new(facts));
}

fn collect_type_facts<'item>(
    expr: &'item Expr,
    state: &State<'item>,
    facts: &mut Vec<TypeFact<'item>>,
) {
    match expr {
        Expr::Parenthesized(parenthesized) => {
            collect_type_facts(&parenthesized.value, state, facts);
        }
        Expr::Call(call)
            if queries::calls::is_binary_intrinsic(call, BinaryIntrinsicFn::And, state) =>
        {
            collect_type_facts(&call.args[0].value, state, facts);
            collect_type_facts(&call.args[1].value, state, facts);
        }
        Expr::Call(call)
            if queries::calls::is_binary_intrinsic(call, BinaryIntrinsicFn::Eq, state) =>
        {
            facts.extend(equality_type_fact(call, state));
        }
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_)
        | Expr::Ident(_) => {}
    }
}

fn equality_type_fact<'item>(call: &'item Call, state: &State<'item>) -> Option<TypeFact<'item>> {
    equality_type_fact_from_exprs(call.id, &call.args[0].value, &call.args[1].value, state).or_else(
        || equality_type_fact_from_exprs(call.id, &call.args[1].value, &call.args[0].value, state),
    )
}

fn equality_type_fact_from_exprs<'item>(
    condition_node_id: u64,
    typeof_expr: &Expr,
    type_expr: &Expr,
    state: &State<'item>,
) -> Option<TypeFact<'item>> {
    if let Expr::Call(typeof_call) = unwrap_parentheses(typeof_expr)
        && queries::calls::is_intrinsic(typeof_call, IntrinsicFn::Typeof, state)
        && let Expr::Ident(ident) = unwrap_parentheses(&typeof_call.args[0].value)
        && let Some(ItemRef::Param(param)) = state.sources.get(&ident.id).copied()
        && let ConstValue::TypeRef(type_) = consts::expr_value(type_expr, state)
    {
        Some(TypeFact {
            param,
            type_,
            condition_node_id,
            subject_span: typeof_expr.span(),
        })
    } else {
        None
    }
}

fn unwrap_parentheses(expr: &Expr) -> &Expr {
    match expr {
        Expr::Parenthesized(parenthesized) => unwrap_parentheses(&parenthesized.value),
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_)
        | Expr::Ident(_) => expr,
    }
}
