mod exprs;
mod type_narrowing;

use self::type_narrowing::TypeNarrowingState;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::modules::Module;
use crate::compiler::parsing::statements::Statement;
use crate::compiler::prelude::PRELUDE_FILE_COUNT;
use crate::compiler::state::{State, TypeFacts};
use crate::utils::indexing::SearchConfig;
use std::rc::Rc;

pub(crate) const FN_CALL_SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: true,
    can_be_parent_node: true,
};
const IDENT_SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: false,
    can_be_parent_node: false,
};

pub(crate) fn index_modules<'item>(modules: &'item [Module], state: &mut State<'item>) {
    let mut state = IndexState::new(state);
    for module in modules {
        index_module_imports(module, &mut state);
    }
    state.inner.imports.consolidate();
    for module in modules {
        index_module_items(module, &mut state);
    }
    state.inner.init_cache();
    index_consts_until_full_registration(modules, &mut state);
    for module in modules {
        index_not_consts(module, &mut state);
    }
}

fn index_module_imports(module: &Module, state: &mut IndexState<'_, '_>) {
    for prelude_file_index in 0..module.file_index.min(PRELUDE_FILE_COUNT) {
        state
            .inner
            .imports
            .register(None, None, module.file_index, prelude_file_index, false);
    }
    for item in &module.items {
        let Item::Import(import) = item else { continue };
        let Some(file_index) = import.imported_file_index else {
            continue;
        };
        let is_pub = import.pub_keyword_span.is_some();
        state.inner.imports.register(
            Some(import.id),
            Some(import.span),
            import.span.file_index,
            file_index,
            is_pub,
        );
    }
}

fn index_module_items<'item>(module: &'item Module, state: &mut IndexState<'_, 'item>) {
    for item in &module.items {
        match item {
            Item::Import(_) | Item::Repeat(_) => (),
            Item::Var(item) => state.inner.items.register(ItemRef::Var(item)),
            Item::Const(item) => state.inner.items.register(ItemRef::Const(item)),
            Item::Struct(item) => state.inner.items.register(ItemRef::Struct(item)),
            Item::Fn(item) => {
                state.inner.items.register(ItemRef::Fn(item));
                for param in &item.params.params {
                    state.inner.items.register(ItemRef::Param(param));
                }
            }
        }
    }
}

fn index_consts_until_full_registration<'item>(
    modules: &'item [Module],
    state: &mut IndexState<'_, 'item>,
) {
    // loop is used to support items referred in function signatures but defined later
    loop {
        state.has_index_changed = false;
        for module in modules {
            index_consts(module, state);
        }
        if !state.has_index_changed {
            break;
        }
    }
    state.phase = IndexPhase::Final;
    for module in modules {
        index_consts(module, state);
    }
}

fn index_consts<'item>(module: &'item Module, state: &mut IndexState<'_, 'item>) {
    for item in &module.items {
        match item {
            Item::Const(item) => exprs::index_expr(&item.value, state),
            Item::Fn(item) => index_fn_const_parts(item, state),
            Item::Import(_) | Item::Var(_) | Item::Struct(_) | Item::Repeat(_) => (),
        }
    }
}

fn index_not_consts<'item>(module: &'item Module, state: &mut IndexState<'_, 'item>) {
    for item in &module.items {
        match item {
            Item::Import(_) | Item::Struct(_) | Item::Const(_) => (),
            Item::Var(item) => exprs::index_expr(&item.default_value, state),
            Item::Fn(item) => index_fn_not_const_parts(item, state),
            Item::Repeat(item) => exprs::index_call(&item.call, state),
        }
    }
}

fn index_fn_const_parts<'item>(fn_: &'item FnDefinition, state: &mut IndexState<'_, 'item>) {
    for param in &fn_.params.params {
        exprs::index_expr(&param.type_, state);
        if let Some(requirement) = &param.requirement {
            exprs::index_expr(&requirement.condition, state);
        }
    }
    if let Some(return_type) = &fn_.return_type {
        exprs::index_expr(return_type, state);
    }
    if fn_.const_keyword_span.is_some()
        && let FnBody::Statements(body) = &fn_.body
    {
        for statement in &body.statements {
            index_statement_refs(statement, state);
        }
    }
}

fn index_fn_not_const_parts<'item>(fn_: &'item FnDefinition, state: &mut IndexState<'_, 'item>) {
    if fn_.const_keyword_span.is_none()
        && let FnBody::Statements(body) = &fn_.body
    {
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

struct IndexState<'state, 'item> {
    inner: &'state mut State<'item>,
    type_narrowing: TypeNarrowingState<'item>,
    phase: IndexPhase,
    has_index_changed: bool,
}

impl<'state, 'item> IndexState<'state, 'item> {
    fn new(state: &'state mut State<'item>) -> Self {
        Self {
            inner: state,
            type_narrowing: TypeNarrowingState::default(),
            phase: IndexPhase::Converging,
            has_index_changed: false,
        }
    }

    fn set_expr_source(&mut self, node_id: u64, source: Option<ItemRef<'item>>) {
        let previous_source = self.inner.sources.get(&node_id).copied();
        if previous_source == source {
            return;
        }
        if let Some(source) = source {
            self.inner.sources.insert(node_id, source);
        } else {
            self.inner.sources.remove(&node_id);
        }
        self.has_index_changed = true;
    }

    fn set_expr_type_facts(&mut self, node_id: u64, facts: Rc<TypeFacts<'item>>) {
        self.has_index_changed |= self.inner.set_expr_type_facts(node_id, facts);
    }
}

#[derive(PartialEq, Eq)]
enum IndexPhase {
    Converging,
    Final,
}

enum CallSource<'item> {
    Found(ItemRef<'item>),
    NotFound,
    Unknown,
}
