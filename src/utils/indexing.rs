use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub(crate) struct ImportIndex {
    imports: Vec<Vec<usize>>, // for each file, ordered by import priority (lowest priority first)
}

impl ImportIndex {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: vec![vec![]; file_count],
        }
    }

    pub(crate) fn register(&mut self, file_index: usize, imported_file_index: usize) {
        self.imports[file_index].push(imported_file_index);
    }

    pub(crate) fn consolidate(&mut self) {
        let direct_imports = self.clone();
        for file_index in 0..self.imports.len() {
            let mut imports = vec![];
            let mut unique_imports = HashSet::new();
            direct_imports.expand_imports(&mut imports, &mut unique_imports, file_index, true);
            imports.reverse();
            self.imports[file_index] = imports;
        }
    }

    fn expand_imports(
        &self,
        imports: &mut Vec<usize>,
        unique_imports: &mut HashSet<usize>,
        new_import: usize,
        is_always_expanded: bool,
    ) {
        if !is_always_expanded && unique_imports.contains(&new_import) {
            return;
        }
        imports.push(new_import);
        unique_imports.insert(new_import);
        for &inner_import in self.imports[new_import].iter().rev() {
            self.expand_imports(imports, unique_imports, inner_import, false);
        }
    }
}

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
    pub(crate) fn search(
        &self,
        key: &str,
        loc: impl NodeRef,
        imports: &ImportIndex,
    ) -> Option<Item> {
        imports.imports[loc.file_index()]
            .iter()
            .filter_map(|&file_index| self.items[file_index].get(key))
            .flatten()
            .rev()
            .find(|&&item| Self::is_item_visible(item, loc))
            .copied()
    }

    fn is_item_visible(item: Item, loc: impl NodeRef) -> bool {
        let is_same_file = loc.file_index() == item.file_index();
        ((is_same_file && item.id() < loc.id()) || (!is_same_file && item.is_public()))
            && item.scope() != loc.scope()
    }
}

pub(crate) trait NodeRef: Clone + Copy {
    fn file_index(&self) -> usize;

    fn id(&self) -> u64;

    fn scope(&self) -> &[u64];

    fn is_public(&self) -> bool;
}
