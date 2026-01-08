use crate::language::items::ItemRef;
use crate::utils::parsing::Span;
use std::collections::HashSet;
use std::mem;

pub(crate) struct Dependencies<'item> {
    item: ItemRef<'item>,
    registered: HashSet<ItemRef<'item>>,
    stack: Vec<Span>,
}

impl<'item> Dependencies<'item> {
    pub(crate) fn new(item: ItemRef<'item>) -> Self {
        Self {
            item,
            registered: HashSet::default(),
            stack: vec![],
        }
    }

    pub(crate) fn register(
        mut self,
        span: Span,
        dependency: ItemRef<'item>,
    ) -> Result<Self, Vec<Span>> {
        self.stack.push(span);
        if dependency == self.item {
            Err(mem::take(&mut self.stack))
        } else {
            self.registered.insert(dependency);
            Ok(self)
        }
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = ItemRef<'item>> {
        self.registered.into_iter()
    }
}
