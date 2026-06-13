#![expect(clippy::multiple_inherent_impl)]

mod consts;
mod types;

// TODO: instead put all internal methods as pub(super) and pub consts and types modules pub(crate)
pub(crate) use consts::ConstValue;
pub(crate) use types::Type;

use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::parsing::items::types::StructDefinition;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct ValueResolver<'item, 'index> {
    indexes: &'index Indexes<'item>,
    scopes: Vec<Scope<'item>>,
}

impl<'item, 'index> ValueResolver<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            indexes,
            scopes: vec![],
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn run_scoped<O>(&mut self, callback: impl FnOnce(&mut Self) -> O) -> O {
        self.enter_scope();
        let output = callback(self);
        self.exit_scope();
        output
    }
}

#[derive(Debug, Default)]
struct Scope<'item> {
    const_values: HashMap<u64, ConstValue<'item>>,
    wildcard_types: HashMap<u64, &'item StructDefinition>,
}
