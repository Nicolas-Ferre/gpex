pub(crate) mod compilerimpl;
pub(crate) mod consts;
pub(crate) mod types;

use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Arg;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::state::State;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types::Type;

// TODO: general comment: to be seen if now we can simplify the name of private functions, because they are not anymore associated to a struct

// TODO: both functions below can be moved in types.rs

pub(crate) fn bind_params_to_args<'item>(
    params: &'item ParamGroup,
    args: &[Arg],
    state: &mut State<'item>,
) -> impl Iterator<Item = (Type<'item>, Type<'item>)> {
    debug_assert_eq!(params.params.len(), args.len());
    params
        .params
        .iter()
        .zip(args)
        .map(|(param, arg)| bind_param_to_arg(param, arg, state))
}

pub(crate) fn bind_param_to_arg<'item>(
    param: &'item Param,
    arg: &Arg,
    state: &mut State<'item>,
) -> (Type<'item>, Type<'item>) {
    let param_type = types::param_type(param, state);
    let arg_type = types::expr_type(&arg.value, state);
    if matches!(param.type_, Expr::Wildcard(_)) {
        types::add_type(param.id, arg_type, state);
    }
    if param.const_mark_span().is_some() {
        let value = consts::expr_const_value(&arg.value, state);
        consts::add_value(param.id, value, state);
    }
    (param_type, arg_type)
}
