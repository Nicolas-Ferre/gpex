use crate::compiler::consts::{self, ConstValue};
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::state::State;

pub(crate) fn is_zero_int(expr: &Expr, state: &State<'_>) -> bool {
    matches!(
        consts::expr_value(expr, state),
        ConstValue::I32(0) | ConstValue::U32(0)
    )
}
