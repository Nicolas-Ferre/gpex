use crate::compiler::consts::{self, ConstValue};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::{BinaryCompilerImplFn, CompilerImplFn};
use crate::compiler::state::State;

pub(crate) fn is_compilerimpl_mul(expr: &Expr, state: &State<'_>) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    matches!(
        state.sources.get(&call.id),
        Some(ItemRef::Fn(source))
            if source.compilerimpl()
                == Some(CompilerImplFn::Binary(BinaryCompilerImplFn::Mul))
    )
}

pub(crate) fn is_zero_int(expr: &Expr, state: &State<'_>) -> bool {
    matches!(
        consts::expr_value(expr, state),
        ConstValue::I32(0) | ConstValue::U32(0)
    )
}
