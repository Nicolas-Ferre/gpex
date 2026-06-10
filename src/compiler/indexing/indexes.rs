use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::prelude::PreludeEndLocation;
use crate::utils::indexing::{ImportIndex, NodeIndex, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::Span;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Indexes<'item> {
    pub(crate) imports: ImportIndex,
    pub(crate) items: NodeIndex<ItemRef<'item>>,
    pub(crate) sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) candidate_sources: HashMap<u64, Vec<ItemRef<'item>>>,
    pub(crate) priv_sources: HashMap<u64, ItemRef<'item>>,
    pub(crate) item_first_refs: HashMap<u64, Span>,
}

impl<'item> Indexes<'item> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: ImportIndex::new(file_count),
            items: NodeIndex::new(file_count),
            sources: HashMap::default(),
            candidate_sources: HashMap::default(),
            priv_sources: HashMap::default(),
            item_first_refs: HashMap::default(),
        }
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
}
