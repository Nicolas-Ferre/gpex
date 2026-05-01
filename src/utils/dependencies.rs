use crate::utils::parsing::span::Span;
use itertools::Itertools;
use std::collections::HashSet;
use std::hash::Hash;
use std::mem;

#[derive(Debug, Clone)]
pub(crate) struct Dependencies<T> {
    registered: HashSet<T>,
    stack: Vec<Span>,
}

impl<T: Eq + Hash + Copy + Ord> Dependencies<T> {
    pub(crate) fn new() -> Self {
        Self {
            registered: HashSet::default(),
            stack: vec![],
        }
    }

    pub(crate) fn enter_item(&mut self, span: Span, dependency: T) -> Result<(), Vec<Span>> {
        if self.stack.contains(&span) {
            return Err(mem::take(&mut self.stack));
        }
        self.stack.push(span);
        self.registered.insert(dependency);
        Ok(())
    }

    pub(crate) fn exit_item(&mut self) {
        self.stack.pop();
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = T> {
        self.registered.iter().copied().sorted_unstable()
    }
}
