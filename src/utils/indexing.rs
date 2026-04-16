use crate::utils::parsing::span::Span;
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

    pub(crate) fn register(
        &mut self,
        import_item_id: Option<u64>,
        span: Option<Span>,
        file_index: usize,
        imported_file_index: usize,
        is_import_pub: bool,
    ) {
        self.imports[file_index].push(ImportItem {
            source_import_id: import_item_id,
            span,
            file_index: imported_file_index,
            is_pub: is_import_pub,
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
                is_pub: true,
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
            is_pub: new_import.is_pub,
            is_used: new_import.is_used,
        });
        unique_file_indexes.insert(new_import.file_index);
        for inner_import in self.imports[new_import.file_index].iter().rev() {
            if inner_import.is_pub {
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
    pub(crate) is_pub: bool,
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = Item> {
        self.items
            .iter()
            .flat_map(HashMap::values)
            .flatten()
            .copied()
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
        params: SearchParams<'_, impl NodeRef>,
        visibility: Visibility,
    ) -> impl Iterator<Item = Item> {
        params.imports.imports[params.location.file_index()]
            .iter()
            .filter_map(|import| self.items[import.file_index].get(params.key))
            .flatten()
            .rev()
            .filter(move |&&item| {
                Self::is_item_visible(item, params.location, visibility, params.config)
            })
            .copied()
    }

    fn is_item_visible(
        item: Item,
        location: impl NodeRef,
        visibility: Visibility,
        config: SearchConfig,
    ) -> bool {
        if item.id() == location.id() {
            return false;
        }
        let is_same_file = location.file_index() == item.file_index();
        if is_same_file && !config.can_be_after && item.id() > location.id() {
            return false;
        }
        let is_pub_item = match visibility {
            Visibility::Enforced => item.is_pub(),
            Visibility::Ignored => true,
        };
        if !is_same_file && !is_pub_item {
            return false;
        }
        let item_scope = item.scope();
        let location_scope = location.scope();
        let is_in_item_own_scope = location_scope.starts_with(item_scope)
            && location_scope.get(item_scope.len()) == Some(&item.id());
        if is_in_item_own_scope {
            return false;
        }
        let is_location_parent = item_scope.len() > location_scope.len()
            && &item_scope[..location_scope.len()] == location_scope;
        if !config.can_be_parent_node && is_location_parent {
            return false;
        }
        true
    }
}

pub(crate) trait NodeRef: Clone + Copy {
    fn file_index(&self) -> usize;

    fn id(&self) -> u64;

    fn scope(&self) -> &[u64];
}

pub(crate) trait ItemNodeRef: NodeRef {
    fn is_pub(&self) -> bool;

    fn key(&self) -> String;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SearchParams<'import, L: NodeRef> {
    pub(crate) key: &'import str,
    pub(crate) location: L,
    pub(crate) imports: &'import ImportIndex,
    pub(crate) config: SearchConfig,
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
