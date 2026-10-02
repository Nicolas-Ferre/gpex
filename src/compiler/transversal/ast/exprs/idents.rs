use crate::utils::indexing::NodeRef;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct Ident {
    pub(crate) id: u64,
    pub(crate) scope: Vec<u64>,
    pub(crate) span: Span,
    pub(crate) slice: String,
}

impl NodeRef for &Ident {
    fn file_index(&self) -> usize {
        self.span.file_index
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn scope(&self) -> &[u64] {
        &self.scope
    }
}
