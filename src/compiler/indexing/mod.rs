use crate::compiler::item_ref::{ArgsMatch, ItemRef};
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::modules::Module;
use crate::compiler::parsing::statements::Statement;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::state::State;
use crate::utils::indexing::{NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::Span;

// TODO: split file

pub(crate) const FN_CALL_SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: true,
    can_be_parent_node: true,
};
const IDENT_SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: false,
    can_be_parent_node: false,
};

pub(crate) fn index_modules<'item>(modules: &'item [Module], state: &mut State<'item>) {
    for module in modules {
        index_module_imports(module, state);
    }
    state.imports.consolidate();
    for module in modules {
        index_module_items(module, state);
    }
    index_consts_until_full_registration(modules, state);
    for module in modules {
        index_not_consts(module, state);
    }
}

fn index_module_imports(module: &Module, state: &mut State<'_>) {
    state
        .imports
        .register(None, None, module.file_index, PRELUDE_FILE_INDEX, false);
    for item in &module.items {
        let Item::Import(import) = item else { continue };
        let Some(file_index) = import.imported_file_index else {
            continue;
        };
        let is_pub = import.pub_keyword_span.is_some();
        state.imports.register(
            Some(import.id),
            Some(import.span),
            import.span.file_index,
            file_index,
            is_pub,
        );
    }
}

fn index_module_items<'item>(module: &'item Module, state: &mut State<'item>) {
    for item in &module.items {
        match item {
            Item::Import(_) | Item::Repeat(_) => (),
            Item::Var(item) => state.items.register(ItemRef::Var(item)),
            Item::Const(item) => state.items.register(ItemRef::Const(item)),
            Item::Struct(item) => state.items.register(ItemRef::Struct(item)),
            Item::Fn(item) => {
                state.items.register(ItemRef::Fn(item));
                for param in &item.params.params {
                    state.items.register(ItemRef::Param(param));
                }
            }
        }
    }
}

fn index_consts_until_full_registration<'item>(modules: &'item [Module], state: &mut State<'item>) {
    state.is_indexing_source_only = true;
    // loop is used to support items referred in function signatures but defined later
    loop {
        let source_count = state.sources.len();
        for module in modules {
            index_consts(module, state);
        }
        if state.sources.len() == source_count {
            break;
        }
    }
    state.is_indexing_source_only = false;
    for module in modules {
        index_consts(module, state);
    }
}

fn index_consts<'item>(module: &'item Module, state: &mut State<'item>) {
    for item in &module.items {
        match item {
            Item::Const(item) => index_expr(&item.value, state),
            Item::Fn(item) => index_fn_const_parts(item, state),
            Item::Import(_) | Item::Var(_) | Item::Struct(_) | Item::Repeat(_) => (),
        }
    }
}

fn index_not_consts<'item>(module: &'item Module, state: &mut State<'item>) {
    for item in &module.items {
        match item {
            Item::Import(_) | Item::Struct(_) | Item::Const(_) => (),
            Item::Var(item) => index_expr(&item.default_value, state),
            Item::Fn(item) => index_fn_not_const_parts(item, state),
            Item::Repeat(item) => index_call(&item.call, state),
        }
    }
}

fn index_fn_const_parts<'item>(fn_: &'item FnDefinition, state: &mut State<'item>) {
    for param in &fn_.params.params {
        index_expr(&param.type_, state);
        if let Some(requirement) = &param.requirement {
            index_expr(&requirement.condition, state);
        }
    }
    if let Some(return_type) = &fn_.return_type {
        index_expr(return_type, state);
    }
    if fn_.const_keyword_span.is_some()
        && let FnBody::Statements(body) = &fn_.body
    {
        for statement in &body.statements {
            index_statement_refs(statement, state);
        }
    }
}

fn index_fn_not_const_parts<'item>(fn_: &'item FnDefinition, state: &mut State<'item>) {
    if fn_.const_keyword_span.is_none()
        && let FnBody::Statements(body) = &fn_.body
    {
        for statement in &body.statements {
            index_statement_refs(statement, state);
        }
    }
}

fn index_statement_refs<'item>(statement: &'item Statement, state: &mut State<'item>) {
    match statement {
        Statement::Return(return_) => index_expr(&return_.value, state),
        Statement::Assignment(assignment) => {
            index_expr(&assignment.assigned, state);
            index_expr(&assignment.value, state);
        }
    }
}

fn index_expr<'item>(expr: &'item Expr, state: &mut State<'item>) {
    match expr {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_) => {}
        Expr::Call(call) => index_call(call, state),
        Expr::Ident(ident) => index_ident(ident, state),
    }
}

fn index_call<'item>(call: &'item Call, state: &mut State<'item>) {
    if state.is_indexing_source_only && state.sources.contains_key(&call.id) {
        return;
    }
    for arg in &call.args {
        index_expr(&arg.value, state); // no-fn-check (recursivity)
    }
    let accessible_search_params = SearchParams {
        key: &call.key(),
        location: call,
        imports: &state.imports,
        config: FN_CALL_SEARCH_CONFIG,
    };
    let accessible_items = state
        .items
        .search(accessible_search_params, Visibility::Enforced)
        .collect::<Vec<_>>();
    let ignored_search_params = SearchParams {
        key: &call.key(),
        location: call,
        imports: &state.imports,
        config: FN_CALL_SEARCH_CONFIG,
    };
    let ignored_items = state
        .items
        .search(ignored_search_params, Visibility::Ignored)
        .collect::<Vec<_>>();
    match search_accessible_call_source(call, &accessible_items, state) {
        CallSource::Found(source) => index_accessible_source(&call, call.span, source, state),
        CallSource::NotFound => {
            if let Some(source) = search_not_accessible_call_source(call, &ignored_items, state) {
                index_not_accessible_source(call.id, source, state);
            } else {
                let candidates = search_candidate_call_sources(&accessible_items);
                index_call_candidates(call.id, candidates, state);
            }
        }
        CallSource::Unknown => {
            let candidates = search_candidate_call_sources(&accessible_items);
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
    items: &[ItemRef<'item>],
    state: &mut State<'item>,
) -> CallSource<'item> {
    for &item in items {
        match item.args_match(&call.args, state) {
            ArgsMatch::Matching => return CallSource::Found(item),
            ArgsMatch::NotMatching => {}
            ArgsMatch::Unknown => return CallSource::Unknown,
        }
    }
    CallSource::NotFound
}

fn search_candidate_call_sources<'item>(items: &[ItemRef<'item>]) -> Vec<ItemRef<'item>> {
    items
        .iter()
        .copied()
        .filter(|item| matches!(item, ItemRef::Fn(_)))
        .collect()
}

fn search_not_accessible_call_source<'item>(
    call: &Call,
    items: &[ItemRef<'item>],
    state: &mut State<'item>,
) -> Option<ItemRef<'item>> {
    items
        .iter()
        .copied()
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

enum CallSource<'item> {
    Found(ItemRef<'item>),
    NotFound,
    Unknown,
}
