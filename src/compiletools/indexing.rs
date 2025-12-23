use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct NodeIndex<Item, const SEARCH_BEFORE: bool> {
    items: Vec<HashMap<String, Vec<Item>>>,
}

impl<Item: NodeRef, const SEARCH_BEFORE: bool> NodeIndex<Item, SEARCH_BEFORE> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            items: vec![HashMap::new(); file_count],
        }
    }

    // It is assumed the item IDs are ordered by location in the file
    pub(crate) fn register(&mut self, key: impl Into<String>, item: Item) {
        self.items[item.file_index()]
            .entry(key.into())
            .or_default()
            .push(item);
    }
}

impl<Item: NodeRef> NodeIndex<Item, false> {
    pub(crate) fn search(&self, key: &str, loc: impl NodeRef) -> Option<Item> {
        self.items[loc.file_index()]
            .get(key)?
            .iter()
            .rev()
            .find(|item| item.id() < loc.id() && item.scope() != loc.scope())
            .copied()
    }
}

pub(crate) trait NodeRef: Clone + Copy {
    fn file_index(&self) -> usize;

    fn id(&self) -> u64;

    fn scope(&self) -> &[u64];
}
