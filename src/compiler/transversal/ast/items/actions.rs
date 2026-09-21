use crate::compiler::transversal::ast::exprs::calls::Call;

#[derive(Debug)]
pub(crate) struct RepeatDefinition {
    pub(crate) call: Call,
}
