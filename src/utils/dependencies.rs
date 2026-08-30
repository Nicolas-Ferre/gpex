use crate::utils::parsing::span::Span;
use itertools::Itertools;
use std::collections::HashSet;
use std::hash::Hash;
use std::mem;

#[derive(Debug, Clone)]
pub(crate) struct Dependencies<Dependency> {
    registered: HashSet<Dependency>,
    stack: Vec<Span>,
}

impl<Dependency: Eq + Hash + Copy + Ord> Dependencies<Dependency> {
    pub(crate) fn new() -> Self {
        Self {
            registered: HashSet::default(),
            stack: vec![],
        }
    }

    pub(crate) fn enter_item(
        &mut self,
        span: Span,
        dependency: Dependency,
    ) -> Result<(), Vec<Span>> {
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = Dependency> {
        self.registered.iter().copied().sorted_unstable()
    }
}
