use crate::compiletools::indexing::NodeIndex;
use crate::language::var_stmt::VarStmt;
use std::collections::HashMap;
use crate::compiletools::parsing::Span;

#[derive(Debug)]
pub(crate) struct Indexes<'a> {
    pub(crate) item_first_ref: HashMap<u64, Span>,
    pub(crate) value_sources: HashMap<u64, &'a VarStmt>,
    pub(crate) values: NodeIndex<'a, VarStmt, false>,
}

impl Indexes<'_> {
    pub(crate) fn new(file_count: usize) -> Self {
        Self {
            item_first_ref: HashMap::default(),
            value_sources: HashMap::default(),
            values: NodeIndex::new(file_count),
        }
    }
}
