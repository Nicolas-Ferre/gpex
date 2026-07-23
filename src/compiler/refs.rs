use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::state::State;

pub(crate) fn is_expr_ref(node: &Expr, state: &State<'_>) -> Option<bool> {
    match node {
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_) => Some(false),
        Expr::Ident(node) => is_ident_ref(node, state),
    }
}

fn is_ident_ref(node: &Ident, state: &State<'_>) -> Option<bool> {
    Some(match state.sources.get(&node.id)? {
        ItemRef::Var(_) => true,
        ItemRef::Param(source) => source.const_mark_span().is_none(),
        ItemRef::Const(_) | ItemRef::Struct(_) => false,
        ItemRef::Fn(_) => unreachable!("identifier should not refer to a function"),
    })
}
