use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Arg;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::state::State;
use crate::compiler::values;
use crate::compiler::values::{ConstValue, consts};
use crate::utils::validation::ValidateError;
use derive_where::derive_where;

pub(crate) fn var_type<'item>(node: &VarDefinition, state: &mut State<'item>) -> Type<'item> {
    expr_type(&node.default_value, state)
}

pub(crate) fn param_type<'item>(node: &'item Param, state: &mut State<'item>) -> Type<'item> {
    if matches!(node.type_, Expr::Wildcard(_)) {
        state.wildcard_type(node.id).unwrap_or(Type::Wildcard(node))
    } else {
        expr_as_type(&node.type_, state)
    }
}

pub(crate) fn fn_type<'item>(node: &FnDefinition, state: &mut State<'item>) -> Type<'item> {
    if let Some(return_type) = node.return_type.as_ref() {
        expr_as_type(return_type, state)
    } else {
        Type::NoReturn
    }
}

pub(crate) fn const_fn_type<'item>(
    node: &'item FnDefinition,
    args: &[Arg],
    state: &mut State<'item>,
) -> Type<'item> {
    state.in_scope(|state_| {
        values::bind_params_to_args(&node.params, args, state_).for_each(drop);
        if let Some(return_type) = node.return_type.as_ref() {
            expr_as_type(return_type, state_)
        } else {
            Type::NoReturn
        }
    })
}

pub(crate) fn expr_type<'item>(node: &Expr, state: &mut State<'item>) -> Type<'item> {
    match node {
        Expr::F32Literal(_) => Type::Struct(state.search_prelude_type("f32")),
        Expr::U32Literal(_) => Type::Struct(state.search_prelude_type("u32")),
        Expr::I32Literal(_) => Type::Struct(state.search_prelude_type("i32")),
        Expr::BoolLiteral(_) => Type::Struct(state.search_prelude_type("bool")),
        Expr::Wildcard(_) => Type::Unknown,
        Expr::Call(node) => source_type(node.id, &node.args, state),
        Expr::Ident(node) => source_type(node.id, &[], state),
    }
}

pub(crate) fn expr_as_type<'item>(node: &Expr, state: &mut State<'item>) -> Type<'item> {
    match consts::expr_const_value(node, state) {
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

fn item_type<'item>(node: ItemRef<'item>, args: &[Arg], state: &mut State<'item>) -> Type<'item> {
    match node {
        ItemRef::Var(node) => var_type(node, state),
        ItemRef::Const(node) => expr_type(&node.value, state),
        ItemRef::Struct(_) => Type::Struct(state.search_prelude_type("typeref")),
        ItemRef::Fn(node) => const_fn_type(node, args, state),
        ItemRef::Param(node) => param_type(node, state),
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
