use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Arg;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::state::State;
use crate::compiler::values::{ConstValue, consts};
use crate::utils::validation::ValidateError;
use derive_where::derive_where;

pub(crate) fn var_type<'item>(var: &VarDefinition, state: &mut State<'item>) -> Type<'item> {
    expr_type(&var.default_value, state)
}

pub(crate) fn fn_type<'item>(fn_: &FnDefinition, state: &mut State<'item>) -> Type<'item> {
    if let Some(return_type) = fn_.return_type.as_ref() {
        expr_as_type(return_type, state)
    } else {
        Type::NoReturn
    }
}

pub(crate) fn const_fn_type<'item>(
    fn_: &'item FnDefinition,
    args: &[Arg],
    state: &mut State<'item>,
) -> Type<'item> {
    state.in_scope(|state_| {
        bind_params_to_args(&fn_.params, args, state_).for_each(drop);
        if let Some(return_type) = fn_.return_type.as_ref() {
            expr_as_type(return_type, state_)
        } else {
            Type::NoReturn
        }
    })
}

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
    let param_type = param_type(param, state);
    let arg_type = expr_type(&arg.value, state);
    if matches!(param.type_, Expr::Wildcard(_)) {
        state.add_wildcard_type(param.id, arg_type);
    }
    if param.const_mark_span().is_some() {
        let value = consts::expr_value(&arg.value, state);
        state.add_const_value(param.id, value);
    }
    (param_type, arg_type)
}

pub(crate) fn param_type<'item>(param: &'item Param, state: &mut State<'item>) -> Type<'item> {
    if matches!(param.type_, Expr::Wildcard(_)) {
        state
            .wildcard_type(param.id)
            .unwrap_or(Type::Wildcard(param))
    } else {
        expr_as_type(&param.type_, state)
    }
}

pub(crate) fn expr_type<'item>(expr: &Expr, state: &mut State<'item>) -> Type<'item> {
    match expr {
        Expr::F32Literal(_) => Type::Struct(state.search_prelude_type("f32")),
        Expr::U32Literal(_) => Type::Struct(state.search_prelude_type("u32")),
        Expr::I32Literal(_) => Type::Struct(state.search_prelude_type("i32")),
        Expr::BoolLiteral(_) => Type::Struct(state.search_prelude_type("bool")),
        Expr::Wildcard(_) => Type::Unknown,
        Expr::Call(call) => source_type(call.id, &call.args, state),
        Expr::Ident(ident) => source_type(ident.id, &[], state),
    }
}

pub(crate) fn expr_as_type<'item>(expr: &Expr, state: &mut State<'item>) -> Type<'item> {
    match consts::expr_value(expr, state) {
        ConstValue::TypeRef(type_) => Type::Struct(type_),
        ConstValue::Param(type_) => Type::Param(type_),
        ConstValue::WildcardType(type_) => Type::Wildcard(type_),
        ConstValue::I32(_)
        | ConstValue::U32(_)
        | ConstValue::F32(_)
        | ConstValue::Bool(_)
        | ConstValue::Unknown
        | ConstValue::RuntimeValue => Type::Unknown,
    }
}

fn source_type<'item>(node_id: u64, args: &[Arg], state: &mut State<'item>) -> Type<'item> {
    match state.sources.get(&node_id).copied() {
        Some(source) => item_type(source, args, state),
        None => Type::Unknown,
    }
}

fn item_type<'item>(item: ItemRef<'item>, args: &[Arg], state: &mut State<'item>) -> Type<'item> {
    match item {
        ItemRef::Var(var) => var_type(var, state),
        ItemRef::Const(const_) => expr_type(&const_.value, state),
        ItemRef::Struct(_) => Type::Struct(state.search_prelude_type("typeref")),
        ItemRef::Fn(fn_) => const_fn_type(fn_, args, state),
        ItemRef::Param(param) => param_type(param, state),
    }
}

#[derive(Debug, Clone, Copy)]
#[derive_where(PartialEq)]
pub(crate) enum Type<'item> {
    Struct(&'item StructDefinition),
    Param(&'item Param),
    Wildcard(&'item Param),
    NoReturn,
    #[derive_where(incomparable)]
    Unknown,
}

impl<'item> Type<'item> {
    pub(crate) fn is_comparable(self) -> bool {
        matches!(self, Self::Struct(_) | Self::Param(_) | Self::Wildcard(_))
    }

    pub(crate) fn name(self) -> Result<String, ValidateError> {
        match self {
            Type::Struct(struct_) => Ok(struct_.name.clone()),
            Type::Param(param) => Ok(param.name.clone()),
            Type::Wildcard(param) => Ok(format!("typeof({})", param.name)),
            Type::NoReturn | Type::Unknown => Err(ValidateError),
        }
    }

    pub(crate) fn struct_ref(self) -> Option<&'item StructDefinition> {
        if let Self::Struct(struct_) = self {
            Some(struct_)
        } else {
            None
        }
    }
}
