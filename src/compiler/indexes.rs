use crate::compiler::constants::ConstValue;
use crate::compiler::dependencies::Dependencies;
use crate::compiletools::indexing::{ImportIndex, NodeIndex, NodeRef};
use crate::compiletools::parsing::Span;
use crate::language::stmts::const_::ConstStmt;
use crate::language::stmts::var::VarStmt;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Indexes<'a> {
    pub(crate) imports: ImportIndex,
    pub(crate) values: NodeIndex<Value<'a>, false>,
    pub(crate) value_sources: HashMap<u64, Value<'a>>,
    pub(crate) item_first_ref: HashMap<u64, Span>,
    pub(crate) const_values: HashMap<u64, ConstValue>,
}

impl Indexes<'_> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            imports: ImportIndex::new(file_count),
            values: NodeIndex::new(file_count),
            value_sources: HashMap::default(),
            item_first_ref: HashMap::default(),
            const_values: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Value<'a> {
    Var(&'a VarStmt),
    Const(&'a ConstStmt),
}

impl Value<'_> {
    pub(crate) fn ident(&self) -> &Span {
        match self {
            Value::Var(node) => &node.ident,
            Value::Const(node) => &node.ident,
        }
    }

    pub(crate) fn dependencies<'a>(
        &self,
        dependencies: Dependencies<'a>,
        indexes: &Indexes<'a>,
    ) -> Result<Dependencies<'a>, Vec<Span>> {
        match self {
            Value::Var(node) => node.dependencies(dependencies, indexes),
            Value::Const(node) => node.dependencies(dependencies, indexes),
        }
    }

    pub(crate) fn const_value(&self, indexes: &Indexes<'_>) -> Option<ConstValue> {
        match self {
            Value::Var(_) => None, // no-coverage (unused for now)
            Value::Const(node) => Some(node.const_value(indexes)),
        }
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match self {
            Value::Var(node) => node.transpile_ref(shader),
            Value::Const(node) => node.transpile_ref(shader, indexes),
        }
    }
}

impl NodeRef for Value<'_> {
    fn file_index(&self) -> usize {
        match self {
            Value::Var(node) => node.ident.file_index,
            Value::Const(node) => node.ident.file_index,
        }
    }

    fn id(&self) -> u64 {
        match self {
            Value::Var(node) => node.id,
            Value::Const(node) => node.id,
        }
    }

    fn scope(&self) -> &[u64] {
        match self {
            Value::Var(node) => &node.scope,
            Value::Const(node) => &node.scope,
        }
    }
}
