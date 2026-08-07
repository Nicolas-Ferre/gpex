use crate::compiler::consts::{self, ConstValue};
use crate::compiler::indexing::{IndexState, exprs};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::queries;
use crate::compiler::state::{State, TypeFactsId};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct TypeNarrowingState {
    type_facts: HashMap<u64, TypeFactsId>,
}

struct TypeFact<'item> {
    item_id: u64,
    type_: &'item StructDefinition,
}

pub(super) fn index_and_args<'item>(call: &'item Call, state: &mut IndexState<'_, 'item>) {
    exprs::index_expr(&call.args[0].value, state);
    let mut facts = HashMap::new();
    collect_type_facts(&call.args[0].value, state.inner, &mut facts);
    let previous_facts = state.type_narrowing.type_facts.clone();
    add_type_facts(facts, state);
    exprs::index_expr(&call.args[1].value, state);
    state.type_narrowing.type_facts = previous_facts;
}

pub(super) fn index_ident<'item>(
    ident: &Ident,
    source: ItemRef<'item>,
    state: &mut IndexState<'_, 'item>,
) {
    if let ItemRef::Param(param) = source
        && let Some(facts_id) = state.type_narrowing.type_facts.get(&param.id).copied()
    {
        state.set_expr_type_facts(ident.id, facts_id);
    }
}

fn add_type_facts<'item>(
    included_item_types: HashMap<u64, Vec<&'item StructDefinition>>,
    state: &mut IndexState<'_, 'item>,
) {
    for (item_id, included_types) in included_item_types {
        let mut facts = state
            .type_narrowing
            .type_facts
            .get(&item_id)
            .map(|id| state.inner.type_facts(*id).clone())
            .unwrap_or_default();
        for included_type in included_types {
            facts.add_included(included_type);
        }
        let facts_id = state.inner.add_type_facts(facts);
        state.type_narrowing.type_facts.insert(item_id, facts_id);
    }
}

fn collect_type_facts<'item>(
    expr: &'item Expr,
    state: &State<'item>,
    facts: &mut HashMap<u64, Vec<&'item StructDefinition>>,
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
            if let Some(fact) = equality_type_fact(call, state) {
                facts.entry(fact.item_id).or_default().push(fact.type_);
            }
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
    equality_type_fact_from_exprs(&call.args[0].value, &call.args[1].value, state)
        .or_else(|| equality_type_fact_from_exprs(&call.args[1].value, &call.args[0].value, state))
}

fn equality_type_fact_from_exprs<'item>(
    typeof_expr: &Expr,
    type_expr: &Expr,
    state: &State<'item>,
) -> Option<TypeFact<'item>> {
    let Expr::Call(typeof_call) = unwrap_parentheses(typeof_expr) else {
        return None;
    };
    if !queries::calls::is_intrinsic(typeof_call, IntrinsicFn::Typeof, state) {
        return None;
    }
    let Expr::Ident(ident) = unwrap_parentheses(&typeof_call.args[0].value) else {
        return None;
    };
    let Some(ItemRef::Param(param)) = state.sources.get(&ident.id) else {
        return None;
    };
    let ConstValue::TypeRef(type_) = consts::expr_value(type_expr, state) else {
        return None;
    };
    Some(TypeFact {
        item_id: param.id,
        type_,
    })
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
