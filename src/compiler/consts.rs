use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;

#[derive(Debug)]
pub(crate) struct ConstChecker<'item, 'index> {
    pub(crate) location: ConstLocation,
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> ConstChecker<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            location: ConstLocation::Other,
            indexes,
        }
    }

    pub(crate) fn is_expr_const(&self, node: &Expr) -> bool {
        match node {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_)
            | Expr::Wildcard(_) => true,
            Expr::Call(node) => self.is_call_const(node),
            Expr::Ident(node) => self.is_ident_const(node),
        }
    }

    pub(crate) fn is_item_const(&self, node: ItemRef<'_>) -> bool {
        match node {
            ItemRef::Var(_) => false,
            ItemRef::Const(_) | ItemRef::Struct(_) => true,
            ItemRef::Fn(node) => node.const_keyword_span.is_some(),
            ItemRef::Param(node) => match self.location {
                ConstLocation::FnSignature | ConstLocation::ConstCallArg => {
                    node.const_mark_span().is_some()
                }
                ConstLocation::ConstFnBody => true,
                ConstLocation::Other => false,
            },
        }
    }

    fn is_ident_const(&self, node: &Ident) -> bool {
        self.indexes
            .sources
            .get(&node.id)
            .is_some_and(|source| self.is_item_const(*source))
    }

    fn is_call_const(&self, node: &Call) -> bool {
        self.indexes.sources.get(&node.id).is_some_and(|source| {
            let are_args_const = source.is_param_constness_ignored()
                || node.args.iter().all(|arg| self.is_expr_const(arg));
            are_args_const && self.is_item_const(*source)
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConstLocation {
    FnSignature,
    ConstFnBody,
    ConstCallArg,
    Other,
}
