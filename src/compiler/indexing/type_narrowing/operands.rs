use crate::compiler::consts::{self, ConstValue};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::IntrinsicFn;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::queries;
use crate::compiler::state::type_facts::TypeFactSubject;
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
            ConstValue::Param(param) => {
                TypeFactOperand::Dynamic(TypeFactSubject::Referenced(param))
            }
            ConstValue::WildcardType(param) => {
                TypeFactOperand::Dynamic(TypeFactSubject::Wildcard(param))
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
        let Expr::Call(call) = expr.unparenthesized() else {
            return false;
        };
        queries::calls::is_intrinsic(call, IntrinsicFn::Typeof, state)
    }

    fn typeof_param(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
        if let Expr::Call(typeof_call) = expr.unparenthesized()
            && queries::calls::is_intrinsic(typeof_call, IntrinsicFn::Typeof, state)
            && let Expr::Ident(ident) = typeof_call.args[0].value.unparenthesized()
            && let Some(ItemRef::Param(param)) = state.sources.get(&ident.id).copied()
        {
            Some(param)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum TypeFactOperand<'item> {
    Concrete(&'item StructDefinition),
    Dynamic(TypeFactSubject<'item>),
}

impl<'item> TypeFactOperand<'item> {
    pub(super) fn from_param(param: &'item Param, state: &State<'item>) -> Self {
        match types::param_type(param, state) {
            Type::Struct(type_) => Self::Concrete(type_),
            type_ @ (Type::Param(_) | Type::Wildcard(_) | Type::NoReturn | Type::Unknown) => {
                Self::Dynamic(TypeFactSubject::from_param_type(param, type_))
            }
        }
    }
}
