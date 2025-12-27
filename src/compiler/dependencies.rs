use crate::compiler::indexes::Value;
use crate::compiletools::parsing::Span;
use std::collections::HashSet;
use std::mem;

pub(crate) struct Dependencies<'a> {
    item: Value<'a>,
    registered: HashSet<Value<'a>>,
    stack: Vec<Span>,
}

impl<'a> Dependencies<'a> {
    pub(crate) fn new(item: Value<'a>) -> Self {
        Self {
            item,
            registered: HashSet::default(),
            stack: vec![],
        }
    }

    pub(crate) fn register(mut self, span: Span, dependency: Value<'a>) -> Result<Self, Vec<Span>> {
        self.stack.push(span);
        if dependency == self.item {
            Err(mem::take(&mut self.stack))
        } else {
            self.registered.insert(dependency);
            Ok(self)
        }
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = Value<'a>> {
        self.registered.into_iter()
    }
}
