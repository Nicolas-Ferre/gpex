use crate::compiler::consts::ConstValue;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::prelude::PreludeTypesEndLocation;
use crate::compiler::types::Type;
use crate::utils::indexing::{ImportIndex, NodeIndex, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::Span;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct State<'item> {
    pub(crate) imports: ImportIndex,
    pub(crate) items: NodeIndex<ItemRef<'item>>,
    pub(crate) sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) candidate_sources: HashMap<u64, Vec<ItemRef<'item>>>,
    pub(crate) priv_sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) item_first_refs: HashMap<u64, Span>,
    pub(crate) is_indexing_source_only: bool,
    scopes: RefCell<Vec<Scope<'item>>>,
    intrinsic_types: HashMap<u64, IntrinsicType>,
}

impl<'item> State<'item> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: ImportIndex::new(file_count),
            items: NodeIndex::new(file_count),
            sources: HashMap::default(),
            candidate_sources: HashMap::default(),
            priv_sources: HashMap::default(),
            item_first_refs: HashMap::default(),
            scopes: RefCell::default(),
            is_indexing_source_only: false,
            intrinsic_types: HashMap::default(),
        }
    }

    pub(crate) fn init_cache(&mut self) {
        self.intrinsic_types = [
            ("i32", IntrinsicType::I32),
            ("u32", IntrinsicType::U32),
            ("f32", IntrinsicType::F32),
            ("bool", IntrinsicType::Bool),
            ("typeref", IntrinsicType::Typeref),
        ]
        .map(|(name, type_)| (self.search_prelude_type(name).id, type_))
        .into();
    }

    pub(crate) fn search_prelude_type(&self, type_name: &str) -> &'item StructDefinition {
        let search_params = SearchParams {
            key: type_name,
            location: PreludeTypesEndLocation,
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

    pub(crate) fn is_intrinsic_type(
        &self,
        type_: Type<'item>,
        intrinsic_type: IntrinsicType,
    ) -> bool {
        type_
            .struct_ref()
            .is_some_and(|type_| self.intrinsic_type(type_) == Some(intrinsic_type))
    }

    pub(crate) fn intrinsic_type(&self, type_: &StructDefinition) -> Option<IntrinsicType> {
        debug_assert!(!self.intrinsic_types.is_empty());
        self.intrinsic_types.get(&type_.id).copied()
    }

    pub(crate) fn wildcard_type(&self, param_id: u64) -> Option<Type<'item>> {
        self.scopes
            .borrow()
            .last()
            .and_then(|scope| scope.wildcard_types.get(&param_id))
            .copied()
    }

    pub(crate) fn const_value(&self, id: u64) -> ConstValue<'item> {
        self.scopes
            .borrow()
            .last()
            .and_then(|scope| scope.const_values.get(&id))
            .cloned()
            .unwrap_or(ConstValue::RuntimeValue)
    }

    pub(crate) fn add_wildcard_type(&self, param_id: u64, type_: Type<'item>) {
        self.scopes
            .borrow_mut()
            .last_mut()
            .unwrap_or_else(|| unreachable!("wildcard parameter type scope should be entered"))
            .wildcard_types
            .insert(param_id, type_);
    }

    pub(crate) fn add_const_value(&self, id: u64, value: ConstValue<'item>) {
        self.scopes
            .borrow_mut()
            .last_mut()
            .unwrap_or_else(|| unreachable!("constant value scope should be entered"))
            .const_values
            .insert(id, value);
    }

    pub(crate) fn in_scope<O>(&self, callback: impl FnOnce(&Self) -> O) -> O {
        self.enter_scope();
        let output = callback(self);
        self.exit_scope();
        output
    }

    pub(crate) fn enter_scope(&self) {
        self.scopes.borrow_mut().push(Scope::default());
    }

    pub(crate) fn exit_scope(&self) {
        self.scopes.borrow_mut().pop();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum IntrinsicType {
    I32,
    U32,
    F32,
    Bool,
    Typeref,
}

#[derive(Debug, Default)]
struct Scope<'item> {
    const_values: HashMap<u64, ConstValue<'item>>,
    wildcard_types: HashMap<u64, Type<'item>>,
}
