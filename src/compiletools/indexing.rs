use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct NodeIndex<'a, Item, const SEARCH_BEFORE: bool> {
    items: Vec<HashMap<String, Vec<&'a Item>>>,
}

impl<'a, Item: Node, const SEARCH_BEFORE: bool> NodeIndex<'a, Item, SEARCH_BEFORE> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            items: vec![HashMap::new(); file_count],
        }
    }

    // It is assumed the item IDs are ordered by location in the file
    pub(crate) fn register(&mut self, key: impl Into<String>, item: &'a Item) {
        self.items[item.file_index()]
            .entry(key.into())
            .or_default()
            .push(item);
    }
}

impl<'a, Item: Node> NodeIndex<'a, Item, false> {
    pub(crate) fn search(&self, key: &str, searched: &impl Node) -> Option<&'a Item> {
        self.items[searched.file_index()]
            .get(key)?
            .iter()
            .rev()
            .find(|item| item.id() < searched.id() && item.scope() != searched.scope())
            .copied()
    }
}

pub(crate) trait Node {
    fn file_index(&self) -> usize;

    fn id(&self) -> u64;

    fn scope(&self) -> &[u64];
}
