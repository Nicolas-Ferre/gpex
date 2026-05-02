use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::modules::Module;
use crate::compiler::parsing::statements::Statement;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::utils::indexing::{NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct Indexer<'item> {
    indexes: Indexes<'item>,
    is_indexation_source_only: bool,
}

impl<'item> Indexer<'item> {
    pub(crate) fn run(modules: &'item [Module]) -> Indexes<'item> {
        let mut indexer = Self {
            indexes: Indexes::new(modules.len()),
            is_indexation_source_only: false,
        };
        indexer.index_modules(modules);
        indexer.indexes
    }

    fn index_modules(&mut self, modules: &'item [Module]) {
        for module in modules {
            self.index_module_imports(module);
        }
        self.indexes.imports.consolidate();
        for module in modules {
            self.index_module_items(module);
        }
        self.index_consts_until_full_registration(modules);
        for module in modules {
            self.index_not_consts(module);
        }
    }

    fn index_module_imports(&mut self, module: &Module) {
        self.indexes
            .imports
            .register(None, None, module.file_index, PRELUDE_FILE_INDEX, false);
        for item in &module.items {
            let Item::Import(import) = item else { continue };
            let Some(file_index) = import.imported_file_index else {
                continue;
            };
            let is_pub = import.pub_keyword_span.is_some();
            self.indexes.imports.register(
                Some(import.id),
                Some(import.span),
                import.span.file_index,
                file_index,
                is_pub,
            );
        }
    }

    fn index_module_items(&mut self, module: &'item Module) {
        for item in &module.items {
            match item {
                Item::Import(_) | Item::Repeat(_) => (),
                Item::Var(item) => self.indexes.items.register(ItemRef::Var(item)),
                Item::Const(item) => self.indexes.items.register(ItemRef::Const(item)),
                Item::Struct(item) => self.indexes.items.register(ItemRef::Struct(item)),
                Item::Fn(item) => {
                    self.indexes.items.register(ItemRef::Fn(item));
                    for param in &item.params.params {
                        self.indexes.items.register(ItemRef::Param(param));
                    }
                }
            }
        }
    }

    fn index_consts_until_full_registration(&mut self, modules: &'item [Module]) {
        self.is_indexation_source_only = true;
        // loop is used to support items referred in function signatures but defined later
        loop {
            let source_count = self.indexes.sources.len();
            for module in modules {
                self.index_consts(module);
            }
            if self.indexes.sources.len() == source_count {
                break;
            }
        }
        self.is_indexation_source_only = false;
        for module in modules {
            self.index_consts(module);
        }
    }

    fn index_consts(&mut self, module: &'item Module) {
        for item in &module.items {
            match item {
                Item::Const(item) => self.index_expr(&item.value),
                Item::Fn(item) => self.index_fn_const_parts(item),
                Item::Import(_) | Item::Var(_) | Item::Struct(_) | Item::Repeat(_) => (),
            }
        }
    }

    fn index_not_consts(&mut self, module: &'item Module) {
        for item in &module.items {
            match item {
                Item::Import(_) | Item::Struct(_) | Item::Const(_) => (),
                Item::Var(item) => self.index_expr(&item.default_value),
                Item::Fn(item) => self.index_fn_not_const_parts(item),
                Item::Repeat(item) => self.index_call(&item.call),
            }
        }
    }

    fn index_fn_const_parts(&mut self, item: &'item FnDefinition) {
        for param in &item.params.params {
            self.index_expr(&param.type_);
        }
        if let Some(return_type) = &item.return_type {
            self.index_expr(return_type);
        }
        if item.const_keyword_span.is_some() {
            for statement in &item.statements {
                self.index_statement_refs(statement);
            }
        }
    }

    fn index_fn_not_const_parts(&mut self, item: &'item FnDefinition) {
        if item.const_keyword_span.is_none() {
            for statement in &item.statements {
                self.index_statement_refs(statement);
            }
        }
    }

    fn index_statement_refs(&mut self, statement: &'item Statement) {
        match statement {
            Statement::Return(statement) => self.index_expr(&statement.value),
            Statement::Assignment(statement) => {
                self.index_expr(&statement.assigned);
                self.index_expr(&statement.value);
            }
        }
    }

    fn index_expr(&mut self, expr: &'item Expr) {
        match expr {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_) => {}
            Expr::Call(expr) => self.index_call(expr),
            Expr::Ident(expr) => self.index_ident(expr),
        }
    }

    fn index_call(&mut self, call: &'item Call) {
        if self.is_indexation_source_only && self.indexes.sources.contains_key(&call.id) {
            return;
        }
        for arg in &call.args {
            self.index_expr(arg); // no-fn-check (recursivity)
        }
        let search_params = SearchParams {
            key: &call.key(),
            location: call,
            imports: &self.indexes.imports,
            config: SearchConfig {
                can_be_after: true,
                can_be_parent_node: true,
            },
        };
        if let Some(source) = self.search_accessible_call_source(call, search_params) {
            self.index_accessible_source(&call, call.span, source);
        } else if let Some(source) = self.search_not_accessible_call_source(call, search_params) {
            self.index_not_accessible_source(call.id, source);
        }
    }

    fn index_ident(&mut self, ident: &'item Ident) {
        if self.is_indexation_source_only && self.indexes.sources.contains_key(&ident.id) {
            return;
        }
        let search_params = SearchParams {
            key: &ident.slice,
            location: ident,
            imports: &self.indexes.imports,
            config: SearchConfig {
                can_be_after: false,
                can_be_parent_node: false,
            },
        };
        let matching_value = self.search_accessible_ident_source(search_params);
        if let Some(source) = matching_value {
            self.index_accessible_source(&ident, ident.span, source);
        } else if let Some(source) = self.search_not_accessible_ident_source(search_params) {
            self.index_not_accessible_source(ident.id, source);
        }
    }

    fn search_accessible_call_source(
        &self,
        call: &Call,
        search_params: SearchParams<'_, &Call>,
    ) -> Option<ItemRef<'item>> {
        self.indexes
            .items
            .search(search_params, Visibility::Enforced)
            .find(|item| item.has_same_param_types_as_args(&call.args, &self.indexes))
    }

    fn search_not_accessible_call_source(
        &self,
        call: &Call,
        search_params: SearchParams<'_, &Call>,
    ) -> Option<ItemRef<'item>> {
        self.indexes
            .items
            .search(search_params, Visibility::Ignored)
            .find(|item| item.has_same_param_types_as_args(&call.args, &self.indexes))
    }

    fn search_accessible_ident_source(
        &self,
        search_params: SearchParams<'_, &Ident>,
    ) -> Option<ItemRef<'item>> {
        self.indexes
            .items
            .search(search_params, Visibility::Enforced)
            .next()
    }

    fn search_not_accessible_ident_source(
        &self,
        search_params: SearchParams<'_, &Ident>,
    ) -> Option<ItemRef<'item>> {
        self.indexes
            .items
            .search(search_params, Visibility::Ignored)
            .find(|source| match source {
                ItemRef::Param(_) => false, // coverage: off (will be covered with https://github.com/Nicolas-Ferre/gpex/issues/115)
                ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Fn(_) => true,
            })
    }

    fn index_accessible_source(
        &mut self,
        ref_: &impl NodeRef,
        ref_span: Span,
        source: ItemRef<'item>,
    ) {
        self.indexes.sources.insert(ref_.id(), source);
        if self.is_indexation_source_only {
            return;
        }
        self.indexes
            .imports
            .mark_as_used(ref_.file_index(), source.file_index());
        self.indexes
            .item_first_refs
            .entry(source.id())
            .or_insert_with(|| ref_span);
    }

    fn index_not_accessible_source(&mut self, node_id: u64, source: ItemRef<'item>) {
        if self.is_indexation_source_only {
            return;
        }
        self.indexes.priv_sources.insert(node_id, source);
    }
}
