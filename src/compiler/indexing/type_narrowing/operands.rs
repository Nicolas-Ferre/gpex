use crate::compiler::consts::{self, ConstValue};
use crate::compiler::indexing::type_narrowing::NarrowedType;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::IntrinsicFn;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::queries;
use crate::compiler::state::{IntrinsicType, State};
use crate::compiler::types::{self, Type};

pub(super) struct ResolvedTypeFactOperand<'item> {
    pub(super) operand: TypeFactOperand<'item>,
    pub(super) is_subject: bool,
}

impl<'item> ResolvedTypeFactOperand<'item> {
    pub(super) fn resolve(operand: &Expr, state: &State<'item>) -> Option<Self> {
        if let Some(param) = Self::typeof_param(operand, state) {
            return Some(ResolvedTypeFactOperand {
                operand: TypeFactOperand::from_param(param, state),
                is_subject: true,
            });
        }
        let resolved_operand = match consts::expr_value(operand, state) {
            ConstValue::TypeRef(type_) => TypeFactOperand::Concrete(type_),
            ConstValue::Param(param) => TypeFactOperand::Dynamic(NarrowedType::Referenced(param)),
            ConstValue::WildcardType(param) => {
                TypeFactOperand::Dynamic(NarrowedType::Wildcard(param))
            }
            ConstValue::I32(_)
            | ConstValue::U32(_)
            | ConstValue::F32(_)
            | ConstValue::Bool(_)
            | ConstValue::Unknown
            | ConstValue::RuntimeValue => return None,
        };
        Some(ResolvedTypeFactOperand {
            operand: resolved_operand,
            is_subject: !Self::is_typeof_expr(operand, state)
                && matches!(resolved_operand, TypeFactOperand::Dynamic(_))
                && state
                    .is_intrinsic_type(types::expr_type(operand, state), IntrinsicType::Typeref),
        })
    }

    fn is_typeof_expr(expr: &Expr, state: &State<'_>) -> bool {
        let Expr::Call(call) = Self::unwrap_parentheses(expr) else {
            return false;
        };
        queries::calls::is_intrinsic(call, IntrinsicFn::Typeof, state)
    }

    fn typeof_param(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
        if let Expr::Call(typeof_call) = Self::unwrap_parentheses(expr)
            && queries::calls::is_intrinsic(typeof_call, IntrinsicFn::Typeof, state)
            && let Expr::Ident(ident) = Self::unwrap_parentheses(&typeof_call.args[0].value)
            && let Some(ItemRef::Param(param)) = state.sources.get(&ident.id).copied()
        {
            Some(param)
        } else {
            None
        }
    }

    fn unwrap_parentheses(expr: &Expr) -> &Expr {
        match expr {
            Expr::Parenthesized(parenthesized) => Self::unwrap_parentheses(&parenthesized.value),
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_)
            | Expr::Wildcard(_)
            | Expr::Call(_)
            | Expr::Ident(_) => expr,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum TypeFactOperand<'item> {
    Concrete(&'item StructDefinition),
    Dynamic(NarrowedType<'item>),
}

impl<'item> TypeFactOperand<'item> {
    pub(super) fn from_param(param: &'item Param, state: &State<'item>) -> Self {
        match types::param_type(param, state) {
            Type::Struct(type_) => Self::Concrete(type_),
            Type::Param(type_param) => Self::Dynamic(NarrowedType::Referenced(type_param)),
            Type::Wildcard(type_param) => Self::Dynamic(NarrowedType::Wildcard(type_param)),
            Type::NoReturn | Type::Unknown => Self::Dynamic(NarrowedType::Wildcard(param)),
        }
    }

    pub(super) fn narrowed_type(self) -> Option<NarrowedType<'item>> {
        match self {
            Self::Dynamic(param) => Some(param),
            Self::Concrete(_) => None,
        }
    }
}
