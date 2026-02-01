use crate::compiler::prelude::PreludeEndLocation;
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::utils::indexing::{ImportIndex, NodeIndex, SearchConfig, Visibility};
use crate::utils::parsing::Span;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct Indexes<'item> {
    pub(crate) imports: ImportIndex,
    pub(crate) items: NodeIndex<ItemRef<'item>, false>,
    pub(crate) types: HashSet<&'item StructDefinition>,
    pub(crate) sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) private_sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) item_first_refs: HashMap<u64, Span>,
}

impl<'item> Indexes<'item> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: ImportIndex::new(file_count),
            items: NodeIndex::new(file_count),
            types: HashSet::default(),
            sources: HashMap::default(),
            private_sources: HashMap::default(),
            item_first_refs: HashMap::default(),
        }
    }

    pub(crate) fn search_prelude_type(&self, type_name: &str) -> &'item StructDefinition {
        let search_config = SearchConfig {
            can_be_after: false,
            can_be_parent_node: false,
        };
        match self.items.search(
            type_name,
            PreludeEndLocation,
            &self.imports,
            Visibility::Enforced,
            search_config,
        ) {
            Some(ItemRef::Struct(item)) => item,
            Some(_) | None => unreachable!("missing `{type_name}` type in prelude"),
        }
    }
}
