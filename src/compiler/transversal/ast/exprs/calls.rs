use crate::compiler::transversal::ast::exprs::Expr;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::span::Span;

pub(crate) const UNARY_NEG_FN_NAME: &str = "__neg__";
pub(crate) const UNARY_NOT_FN_NAME: &str = "__not__";
pub(crate) const UNARY_FN_NAMES: &[&str] = &[UNARY_NEG_FN_NAME, UNARY_NOT_FN_NAME];

#[derive(Debug)]
pub(crate) struct Call {
    pub(crate) id: u64,
    pub(crate) scope: Vec<u64>,
    pub(crate) span: Span,
    pub(crate) name: String,
    pub(crate) args: Vec<Arg>,
}

impl NodeRef for &Call {
    fn file_index(&self) -> usize {
        self.span.file_index
    }

    fn id(&self) -> u64 {
        self.id
    }

    // coverage: off (unused because function can be called in itself)
    fn scope(&self) -> &[u64] {
        &self.scope
    }
    // coverage: on
}

impl Call {
    pub(crate) fn key(&self) -> String {
        format!("{}({})", self.name, self.args.len())
    }
}

#[derive(Debug)]
pub(crate) struct Arg {
    pub(crate) name: Option<String>,
    pub(crate) name_span: Option<Span>,
    pub(crate) value: Expr,
}
