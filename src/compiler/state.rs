use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::prelude::PreludeEndLocation;
use crate::compiler::transpilation::SpecializedFn;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types::Type;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{ImportIndex, NodeIndex, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::Span;
use crate::utils::reading::ReadFile;
use crate::utils::validation::ValidateContext;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// TODO: too many fields ?

#[derive(Debug)]
pub(crate) struct State<'item> {
    pub(crate) imports: ImportIndex,
    pub(crate) items: NodeIndex<ItemRef<'item>>,
    pub(crate) sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) candidate_sources: HashMap<u64, Vec<ItemRef<'item>>>,
    pub(crate) priv_sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) item_first_refs: HashMap<u64, Span>,
    pub(crate) compilerimpl_types: HashMap<u64, CompilerImplType>, // TODO: use anywhere possible
    pub(crate) validation_context: ValidateContext<'item>,
    pub(crate) scopes: Vec<Scope<'item>>,
    pub(crate) is_indexing_source_only: bool,
    pub(crate) const_mark_span: Option<Span>,
    pub(crate) param_constness: ParamConstness,
    pub(crate) dependencies: Dependencies<ItemRef<'item>>,
    pub(crate) shader: String,
    pub(crate) specialized_fns: HashMap<SpecializedFn<'item>, usize>,
    pub(crate) transpiled_specialized_fn_indexes: HashSet<usize>,
}

impl<'item> State<'item> {
    pub(crate) fn new(files: &'item [ReadFile], root_path: &'item Path) -> Self {
        Self {
            imports: ImportIndex::new(files.len()),
            items: NodeIndex::new(files.len()),
            sources: HashMap::default(),
            candidate_sources: HashMap::default(),
            priv_sources: HashMap::default(),
            item_first_refs: HashMap::default(),
            compilerimpl_types: HashMap::default(),
            validation_context: ValidateContext::new(files, root_path),
            scopes: vec![],
            is_indexing_source_only: false,
            const_mark_span: None,
            param_constness: ParamConstness::ExplicitOnly,
            dependencies: Dependencies::new(),
            shader: String::new(),
            specialized_fns: HashMap::default(),
            transpiled_specialized_fn_indexes: HashSet::default(),
        }
    }

    pub(crate) fn init_cache(&mut self) {
        self.compilerimpl_types = [
            ("i32", CompilerImplType::I32),
            ("u32", CompilerImplType::U32),
            ("f32", CompilerImplType::F32),
            ("bool", CompilerImplType::Bool),
            ("typeref", CompilerImplType::Typeref),
        ]
        .map(|(name, type_)| (self.search_prelude_type(name).id, type_))
        .into();
    }

    pub(crate) fn search_prelude_type(&self, type_name: &str) -> &'item StructDefinition {
        let search_params = SearchParams {
            key: type_name,
            location: PreludeEndLocation,
            imports: &self.imports,
            config: SearchConfig {
                can_be_after: false,
                can_be_parent_node: false,
            },
        };
        let matching_struct = self
            .items
            .search(search_params, Visibility::Enforced)
            .next();
        match matching_struct {
            Some(ItemRef::Struct(item)) => item,
            Some(_) | None => unreachable!("missing `{type_name}` type in prelude"),
        }
    }

    pub(crate) fn run_scoped<O>(&mut self, callback: impl FnOnce(&mut Self) -> O) -> O {
        self.enter_scope();
        let output = callback(self);
        self.exit_scope();
        output
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }
}

#[derive(Debug, Default)]
pub(crate) struct Scope<'item> {
    pub(crate) const_values: HashMap<u64, ConstValue<'item>>,
    pub(crate) wildcard_types: HashMap<u64, Type<'item>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ParamConstness {
    ExplicitOnly,
    All,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum CompilerImplType {
    I32,
    U32,
    F32,
    Bool,
    Typeref,
}
