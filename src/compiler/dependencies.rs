use crate::language::items::ItemRef;
use crate::utils::parsing::Span;
use std::collections::HashSet;
use std::mem;

pub(crate) struct Dependencies<'items> {
    item: ItemRef<'items>,
    registered: HashSet<ItemRef<'items>>,
    stack: Vec<Span>,
}

impl<'items> Dependencies<'items> {
    pub(crate) fn new(item: ItemRef<'items>) -> Self {
        Self {
            item,
            registered: HashSet::default(),
            stack: vec![],
        }
    }

    pub(crate) fn register(
        mut self,
        span: Span,
        dependency: ItemRef<'items>,
    ) -> Result<Self, Vec<Span>> {
        self.stack.push(span);
        if dependency == self.item {
            Err(mem::take(&mut self.stack))
        } else {
            self.registered.insert(dependency);
            Ok(self)
        }
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = ItemRef<'items>> {
        self.registered.into_iter()
    }
}
