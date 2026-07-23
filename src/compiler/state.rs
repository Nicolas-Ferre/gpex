use crate::compiler::item_ref::ItemRef;
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
use crate::{Log, LogLocation};
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
    pub(crate) validation: ValidateContext<'item>,
    pub(crate) is_indexing_source_only: bool,
    pub(crate) const_mark_span: Option<Span>,
    pub(crate) param_constness: ParamConstness,
    pub(crate) dependencies: Dependencies<ItemRef<'item>>,
    pub(crate) shader: String,
    pub(crate) specialized_fns: HashMap<SpecializedFn<'item>, usize>,
    pub(crate) transpiled_specialized_fn_indexes: HashSet<usize>,
    scopes: Vec<Scope<'item>>,
    compilerimpl_types: HashMap<u64, CompilerImplType>,
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
            validation: ValidateContext::new(files, root_path),
            scopes: vec![],
            is_indexing_source_only: false,
            const_mark_span: None,
            param_constness: ParamConstness::ExplicitOnly,
            dependencies: Dependencies::new(),
            shader: String::new(),
            specialized_fns: HashMap::default(),
            transpiled_specialized_fn_indexes: HashSet::default(),
            compilerimpl_types: HashMap::default(),
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

    pub(crate) fn is_compilerimpl_type(
        &self,
        type_: Type<'item>,
        compilerimpl_type: CompilerImplType,
    ) -> bool {
        type_
            .struct_ref()
            .is_some_and(|type_| self.compilerimpl_type(type_) == Some(compilerimpl_type))
    }

    pub(crate) fn compilerimpl_type(&self, type_: &StructDefinition) -> Option<CompilerImplType> {
        self.compilerimpl_types.get(&type_.id).copied()
    }

    pub(crate) fn span_location(&self, span: Span) -> LogLocation {
        self.validation.location(span)
    }

    pub(crate) fn wildcard_type(&self, param_id: u64) -> Option<Type<'item>> {
        self.scopes
            .last()
            .and_then(|scope| scope.wildcard_types.get(&param_id))
            .copied()
    }

    pub(crate) fn const_value(&self, id: u64) -> ConstValue<'item> {
        self.scopes
            .last()
            .and_then(|scope| scope.const_values.get(&id))
            .cloned()
            .unwrap_or(ConstValue::RuntimeValue)
    }

    pub(crate) fn add_log(&mut self, log: Log) {
        self.validation.logs.push(log);
    }

    pub(crate) fn add_wildcard_type(&mut self, param_id: u64, type_: Type<'item>) {
        self.scopes
            .last_mut()
            .unwrap_or_else(|| unreachable!("wildcard parameter type scope should be entered"))
            .wildcard_types
            .insert(param_id, type_);
    }

    pub(crate) fn add_const_value(&mut self, id: u64, value: ConstValue<'item>) {
        self.scopes
            .last_mut()
            .unwrap_or_else(|| unreachable!("constant value scope should be entered"))
            .const_values
            .insert(id, value);
    }

    pub(crate) fn in_scope<O>(&mut self, callback: impl FnOnce(&mut Self) -> O) -> O {
        self.enter_scope();
        let output = callback(self);
        self.exit_scope();
        output
    }

    pub(crate) fn with_param_constness<O>(
        &mut self,
        param_constness: ParamConstness,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let previous_param_constness = self.param_constness;
        self.param_constness = param_constness;
        let output = callback(self);
        self.param_constness = previous_param_constness;
        output
    }

    pub(crate) fn with_const_mark_span<O>(
        &mut self,
        span: Option<Span>,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let previous_const_mark_span = self.const_mark_span;
        self.const_mark_span = span;
        let output = callback(self);
        self.const_mark_span = previous_const_mark_span;
        output
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }
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

#[derive(Debug, Default)]
struct Scope<'item> {
    pub(crate) const_values: HashMap<u64, ConstValue<'item>>,
    pub(crate) wildcard_types: HashMap<u64, Type<'item>>,
}
