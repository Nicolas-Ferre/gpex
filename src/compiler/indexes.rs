use crate::compiler::constants::Constant;
use crate::language::items::ItemRef;
use crate::utils::indexing::{ImportIndex, NodeIndex};
use crate::utils::parsing::Span;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Indexes<'items> {
    pub(crate) imports: ImportIndex,
    pub(crate) items: NodeIndex<ItemRef<'items>, false>,
    pub(crate) sources: HashMap<u64, ItemRef<'items>>,
    pub(crate) private_sources: HashMap<u64, ItemRef<'items>>,
    pub(crate) item_first_refs: HashMap<u64, Span>,
    pub(crate) constants: HashMap<u64, Constant>,
}

impl Indexes<'_> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: ImportIndex::new(file_count),
            items: NodeIndex::new(file_count),
            sources: HashMap::default(),
            private_sources: HashMap::default(),
            item_first_refs: HashMap::default(),
            constants: HashMap::default(),
        }
    }
}
