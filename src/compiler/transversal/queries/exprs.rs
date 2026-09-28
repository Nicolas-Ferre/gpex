use crate::compiler::transversal::ast::exprs::Expr;
use crate::compiler::transversal::state::State;
use crate::compiler::transversal::values::consts::{self, ConstValue};

pub(crate) fn is_zero_int(expr: &Expr, state: &State<'_>) -> bool {
    matches!(
        consts::expr_value(expr, state),
        ConstValue::I32(0) | ConstValue::U32(0)
    )
}
