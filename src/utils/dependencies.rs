use crate::utils::parsing::Span;
use std::collections::HashSet;
use std::hash::Hash;
use std::mem;

pub(crate) struct Dependencies<T> {
    registered: HashSet<T>,
    stack: Vec<Span>,
}

impl<T: Eq + Hash + Copy> Dependencies<T> {
    pub(crate) fn new() -> Self {
        Self {
            registered: HashSet::default(),
            stack: vec![],
        }
    }

    pub(crate) fn register(
        mut self,
        span: Span,
        dependency: T,
        mut inner_dependencies: impl FnMut(Self) -> Result<Self, Vec<Span>>,
    ) -> Result<Self, Vec<Span>> {
        if self.stack.contains(&span) {
            return Err(mem::take(&mut self.stack));
        }
        self.stack.push(span);
        self.registered.insert(dependency);
        let mut dependencies = inner_dependencies(self)?;
        dependencies.stack.pop();
        Ok(dependencies)
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = T> {
        self.registered.into_iter()
    }
}
