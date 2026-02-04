use crate::utils::parsing::Span;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::iter;

#[derive(Debug, Clone)]
pub(crate) struct ImportIndex {
    imports: Vec<Vec<ImportItem>>, // for each file, ordered by import priority (lowest priority first)
}

impl ImportIndex {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: vec![vec![]; file_count],
        }
    }

    pub(crate) fn imports(&self, file_index: usize) -> &[ImportItem] {
        &self.imports[file_index]
    }

    pub(crate) fn register(
        &mut self,
        import_item_id: Option<u64>,
        span: Option<Span>,
        file_index: usize,
        imported_file_index: usize,
        is_import_public: bool,
    ) {
        self.imports[file_index].push(ImportItem {
            source_import_id: import_item_id,
            span,
            file_index: imported_file_index,
            is_public: is_import_public,
            is_used: false,
        });
    }

    pub(crate) fn is_used(&self, file_index: usize, import_id: u64) -> bool {
        self.imports[file_index]
            .iter()
            .filter(|item| item.source_import_id == Some(import_id))
            .any(|item| item.is_used)
    }

    pub(crate) fn mark_as_used(&mut self, file_index: usize, imported_file_index: usize) {
        if let Some(import) = self.imports[file_index]
            .iter_mut()
            .find(|import| import.file_index == imported_file_index)
        {
            import.is_used = true;
        }
    }

    pub(crate) fn consolidate(&mut self) {
        let direct_imports = self.clone();
        for file_index in 0..self.imports.len() {
            let mut imports = vec![ImportItem {
                source_import_id: None,
                span: None,
                file_index,
                is_public: true,
                is_used: false,
            }];
            let mut unique_file_indexes = iter::once(file_index).collect();
            for inner_import in self.imports[file_index].iter().rev() {
                direct_imports.expand_imports(
                    &mut imports,
                    &mut unique_file_indexes,
                    inner_import,
                    inner_import.source_import_id,
                );
            }
            imports.reverse();
            self.imports[file_index] = imports;
        }
    }

    fn expand_imports(
        &self,
        imports: &mut Vec<ImportItem>,
        unique_file_indexes: &mut HashSet<usize>,
        new_import: &ImportItem,
        source_import_id: Option<u64>,
    ) {
        if unique_file_indexes.contains(&new_import.file_index) {
            return;
        }
        imports.push(ImportItem {
            source_import_id,
            span: new_import.span,
            file_index: new_import.file_index,
            is_public: new_import.is_public,
            is_used: new_import.is_used,
        });
        unique_file_indexes.insert(new_import.file_index);
        for inner_import in self.imports[new_import.file_index].iter().rev() {
            if inner_import.is_public {
                self.expand_imports(imports, unique_file_indexes, inner_import, source_import_id);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ImportItem {
    pub(crate) source_import_id: Option<u64>,
    pub(crate) span: Option<Span>,
    pub(crate) file_index: usize,
    pub(crate) is_public: bool,
    pub(crate) is_used: bool,
}

#[derive(Debug)]
pub(crate) struct NodeIndex<Item> {
    items: Vec<HashMap<String, Vec<Item>>>,
}

impl<Item: ItemNodeRef> NodeIndex<Item> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            items: vec![HashMap::new(); file_count],
        }
    }

    // It is assumed the item IDs are ordered by location in the file
    pub(crate) fn register(&mut self, item: Item) {
        self.items[item.file_index()]
            .entry(item.key())
            .or_default()
            .push(item);
    }

    pub(crate) fn iter_by_key(&self, key: &str) -> impl Iterator<Item = Item> {
        self.items
            .iter()
            .filter_map(|items| items.get(key))
            .flatten()
            .copied()
    }

    pub(crate) fn search(
        &self,
        key: &str,
        location: impl NodeRef,
        imports: &ImportIndex,
        visibility: Visibility,
        config: SearchConfig,
    ) -> Option<Item> {
        imports.imports[location.file_index()]
            .iter()
            .filter_map(|import| self.items[import.file_index].get(key))
            .flatten()
            .rev()
            .find(|&&item| Self::is_item_visible(item, location, visibility, config))
            .copied()
    }

    fn is_item_visible(
        item: Item,
        location: impl NodeRef,
        visibility: Visibility,
        config: SearchConfig,
    ) -> bool {
        let is_same_file = location.file_index() == item.file_index();
        let is_visible_locally = config.can_be_after || item.id() <= location.id();
        let is_item_public = match visibility {
            Visibility::Enforced => item.is_public(),
            Visibility::Ignored => true,
        };
        ((is_same_file && is_visible_locally) || (!is_same_file && is_item_public))
            && (config.can_be_parent_node || item.scope() != location.scope())
    }
}

pub(crate) trait NodeRef: Clone + Copy {
    fn file_index(&self) -> usize;

    fn id(&self) -> u64;

    fn scope(&self) -> &[u64];
}

pub(crate) trait ItemNodeRef: NodeRef {
    fn is_public(&self) -> bool;

    fn key(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
    Enforced,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SearchConfig {
    pub(crate) can_be_after: bool,
    pub(crate) can_be_parent_node: bool,
}
