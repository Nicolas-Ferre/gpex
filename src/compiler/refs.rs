use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::idents::Ident;

#[derive(Debug)]
pub(crate) struct RefChecker<'item, 'index> {
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> RefChecker<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self { indexes }
    }

    pub(crate) fn is_expr_ref(&self, node: &Expr) -> Option<bool> {
        match node {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_)
            | Expr::Call(_) => Some(false),
            Expr::Ident(node) => self.is_ident_ref(node),
        }
    }

    fn is_ident_ref(&self, node: &Ident) -> Option<bool> {
        Some(match self.indexes.sources.get(&node.id)? {
            ItemRef::Var(_) => true,
            ItemRef::Param(source) => source.const_mark_span().is_none(),
            ItemRef::Const(_) | ItemRef::Struct(_) => false,
            ItemRef::Fn(_) => unreachable!("identifier should not refer to a function"),
        })
    }
}
