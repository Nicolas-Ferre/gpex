use crate::compiler::indexing::{CallSource, FN_CALL_SEARCH_CONFIG, IDENT_SEARCH_CONFIG};
use crate::compiler::item_ref::{ArgsMatch, ItemRef};
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::state::State;
use crate::utils::indexing::{NodeRef, SearchParams, Visibility};
use crate::utils::parsing::span::Span;

pub(super) fn index_expr<'item>(expr: &'item Expr, state: &mut State<'item>) {
    match expr {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_) => {}
        Expr::Call(call) => index_call(call, state),
        Expr::Ident(ident) => index_ident(ident, state),
        Expr::Parenthesized(parenthesized) => index_expr(&parenthesized.value, state),
    }
}

pub(super) fn index_call<'item>(call: &'item Call, state: &mut State<'item>) {
    if state.is_indexing_source_only && state.sources.contains_key(&call.id) {
        return;
    }
    for arg in &call.args {
        index_expr(&arg.value, state); // no-fn-check (recursivity)
    }
    let search_params = SearchParams {
        key: &call.key(),
        location: call,
        imports: &state.imports,
        config: FN_CALL_SEARCH_CONFIG,
    };
    match search_accessible_call_source(call, search_params, state) {
        CallSource::Found(source) => index_accessible_source(&call, call.span, source, state),
        CallSource::NotFound => {
            if let Some(source) = search_not_accessible_call_source(call, search_params, state) {
                index_not_accessible_source(call.id, source, state);
            } else {
                let candidates = search_candidate_call_sources(search_params, state);
                index_call_candidates(call.id, candidates, state);
            }
        }
        CallSource::Unknown => {
            let candidates = search_candidate_call_sources(search_params, state);
            index_call_candidates(call.id, candidates, state);
        }
    }
}

fn index_ident<'item>(ident: &'item Ident, state: &mut State<'item>) {
    if state.is_indexing_source_only && state.sources.contains_key(&ident.id) {
        return;
    }
    let search_params = SearchParams {
        key: &ident.slice,
        location: ident,
        imports: &state.imports,
        config: IDENT_SEARCH_CONFIG,
    };
    let matching_value = search_accessible_ident_source(search_params, state);
    if let Some(source) = matching_value {
        index_accessible_source(&ident, ident.span, source, state);
    } else if let Some(source) = search_not_accessible_ident_source(search_params, state) {
        index_not_accessible_source(ident.id, source, state);
    }
}

fn search_accessible_call_source<'item>(
    call: &Call,
    search_params: SearchParams<'_, &Call>,
    state: &State<'item>,
) -> CallSource<'item> {
    for item in state.items.search(search_params, Visibility::Enforced) {
        match item.args_match(&call.args, state) {
            ArgsMatch::Matching => return CallSource::Found(item),
            ArgsMatch::NotMatching => {}
            ArgsMatch::Unknown => return CallSource::Unknown,
        }
    }
    CallSource::NotFound
}

fn search_candidate_call_sources<'item>(
    search_params: SearchParams<'_, &Call>,
    state: &State<'item>,
) -> Vec<ItemRef<'item>> {
    state
        .items
        .search(search_params, Visibility::Enforced)
        .filter(|item| matches!(item, ItemRef::Fn(_)))
        .collect()
}

fn search_not_accessible_call_source<'item>(
    call: &Call,
    search_params: SearchParams<'_, &Call>,
    state: &State<'item>,
) -> Option<ItemRef<'item>> {
    state
        .items
        .search(search_params, Visibility::Ignored)
        .find(|item| item.args_match(&call.args, state) == ArgsMatch::Matching)
}

fn search_accessible_ident_source<'item>(
    search_params: SearchParams<'_, &Ident>,
    state: &State<'item>,
) -> Option<ItemRef<'item>> {
    state
        .items
        .search(search_params, Visibility::Enforced)
        .next()
}

fn search_not_accessible_ident_source<'item>(
    search_params: SearchParams<'_, &Ident>,
    state: &State<'item>,
) -> Option<ItemRef<'item>> {
    state
        .items
        .search(search_params, Visibility::Ignored)
        .find(|source| match source {
            ItemRef::Param(_) => false,
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Fn(_) => true,
        })
}

fn index_call_candidates<'item>(
    node_id: u64,
    candidates: Vec<ItemRef<'item>>,
    state: &mut State<'item>,
) {
    if state.is_indexing_source_only {
        return;
    }
    if !candidates.is_empty() {
        index_candidate_sources(node_id, candidates, state);
    }
}

fn index_accessible_source<'item>(
    node: &impl NodeRef,
    ref_span: Span,
    source: ItemRef<'item>,
    state: &mut State<'item>,
) {
    state.sources.insert(node.id(), source);
    if state.is_indexing_source_only {
        return;
    }
    state
        .imports
        .mark_as_used(node.file_index(), source.file_index());
    state
        .item_first_refs
        .entry(source.id())
        .or_insert_with(|| ref_span);
}

fn index_candidate_sources<'item>(
    node_id: u64,
    sources: Vec<ItemRef<'item>>,
    state: &mut State<'item>,
) {
    state.candidate_sources.insert(node_id, sources);
}

fn index_not_accessible_source<'item>(
    node_id: u64,
    source: ItemRef<'item>,
    state: &mut State<'item>,
) {
    if state.is_indexing_source_only {
        return;
    }
    state.priv_sources.insert(node_id, source);
}
