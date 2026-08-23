#![expect(clippy::wildcard_enum_match_arm, reason = "opt-in is preferred here")]

use crate::compiler::consts::{ConstValue, HashableF32};
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{
    BinaryIntrinsicFn, FnDefinition, IntrinsicFn, UnaryIntrinsicFn,
};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::state::State;
use crate::compiler::types;
use crate::compiler::types::Type;

pub(super) fn call_value<'item>(
    call: &Call,
    source: &'item FnDefinition,
    state: &State<'item>,
) -> ConstValue<'item> {
    for param in &source.params.params {
        if matches!(
            state.const_value(param.id),
            ConstValue::Param(_) | ConstValue::WildcardType(_)
        ) {
            return ConstValue::Unknown;
        }
    }
    match source.intrinsic() {
        Some(IntrinsicFn::Binary(fn_)) => fn_binary_value(source, fn_, state),
        Some(IntrinsicFn::Unary(fn_)) => fn_unary_value(source, fn_, state),
        Some(IntrinsicFn::MulAdd) => mul_add_value(source, state),
        Some(IntrinsicFn::Sizeof) => sizeof_value(source, state),
        Some(IntrinsicFn::Typeof) => typeof_value(call, state),
        None => unreachable!("not implemented `{}` constant GPU function", source.name),
    }
}

pub(super) fn decisive_left_value<'item>(
    call: &Call,
    source: &FnDefinition,
    state: &State<'item>,
) -> Option<ConstValue<'item>> {
    let intrinsic = source.intrinsic()?;
    let left = super::expr_value(&call.args.first()?.value, state);
    match (intrinsic, left) {
        (IntrinsicFn::Binary(BinaryIntrinsicFn::And), ConstValue::Bool(false)) => {
            Some(ConstValue::Bool(false))
        }
        (IntrinsicFn::Binary(BinaryIntrinsicFn::Or), ConstValue::Bool(true)) => {
            Some(ConstValue::Bool(true))
        }
        _ => None,
    }
}

fn fn_binary_value<'item>(
    source: &'item FnDefinition,
    fn_: BinaryIntrinsicFn,
    state: &State<'item>,
) -> ConstValue<'item> {
    let left = state.const_value(source.params.params[0].id);
    let right = state.const_value(source.params.params[1].id);
    match (left, right) {
        (ConstValue::I32(left), ConstValue::I32(right)) => fn_binary_i32_value(left, right, fn_),
        (ConstValue::U32(left), ConstValue::U32(right)) => fn_binary_u32_value(left, right, fn_),
        (ConstValue::F32(left), ConstValue::F32(right)) => fn_binary_f32_value(left, right, fn_),
        (ConstValue::Bool(left), ConstValue::Bool(right)) => fn_binary_bool_value(left, right, fn_),
        (ConstValue::TypeRef(left), ConstValue::TypeRef(right)) => {
            fn_binary_typeref_value(left, right, fn_)
        }
        _ => unreachable!("not implemented `{}` constant GPU function", source.name),
    }
}

fn fn_binary_i32_value<'item>(left: i32, right: i32, fn_: BinaryIntrinsicFn) -> ConstValue<'item> {
    match fn_ {
        BinaryIntrinsicFn::Add => ConstValue::I32(left.wrapping_add(right)),
        BinaryIntrinsicFn::Sub => ConstValue::I32(left.wrapping_sub(right)),
        BinaryIntrinsicFn::Mul => ConstValue::I32(left.wrapping_mul(right)),
        BinaryIntrinsicFn::Div => ConstValue::I32(if right == 0 {
            left
        } else {
            left.wrapping_div(right)
        }),
        BinaryIntrinsicFn::Mod => ConstValue::I32(if right == 0 {
            0
        } else {
            left.wrapping_rem(right)
        }),
        BinaryIntrinsicFn::Eq => ConstValue::Bool(left == right),
        BinaryIntrinsicFn::Ne => ConstValue::Bool(left != right),
        BinaryIntrinsicFn::Lt => ConstValue::Bool(left < right),
        BinaryIntrinsicFn::Le => ConstValue::Bool(left <= right),
        BinaryIntrinsicFn::Gt => ConstValue::Bool(left > right),
        BinaryIntrinsicFn::Ge => ConstValue::Bool(left >= right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `i32`"),
    }
}

fn fn_binary_u32_value<'item>(left: u32, right: u32, fn_: BinaryIntrinsicFn) -> ConstValue<'item> {
    match fn_ {
        BinaryIntrinsicFn::Add => ConstValue::U32(left.wrapping_add(right)),
        BinaryIntrinsicFn::Sub => ConstValue::U32(left.wrapping_sub(right)),
        BinaryIntrinsicFn::Mul => ConstValue::U32(left.wrapping_mul(right)),
        BinaryIntrinsicFn::Div => ConstValue::U32(if right == 0 {
            left
        } else {
            left.wrapping_div(right)
        }),
        BinaryIntrinsicFn::Mod => ConstValue::U32(if right == 0 {
            0
        } else {
            left.wrapping_rem(right)
        }),
        BinaryIntrinsicFn::Eq => ConstValue::Bool(left == right),
        BinaryIntrinsicFn::Ne => ConstValue::Bool(left != right),
        BinaryIntrinsicFn::Lt => ConstValue::Bool(left < right),
        BinaryIntrinsicFn::Le => ConstValue::Bool(left <= right),
        BinaryIntrinsicFn::Gt => ConstValue::Bool(left > right),
        BinaryIntrinsicFn::Ge => ConstValue::Bool(left >= right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `u32`"),
    }
}

#[expect(clippy::float_cmp, reason = "same behavior as GPEx runtime")]
fn fn_binary_f32_value<'item>(
    left: HashableF32,
    right: HashableF32,
    fn_: BinaryIntrinsicFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryIntrinsicFn::Add => ConstValue::F32(HashableF32(left.0 + right.0)),
        BinaryIntrinsicFn::Sub => ConstValue::F32(HashableF32(left.0 - right.0)),
        BinaryIntrinsicFn::Mul => ConstValue::F32(HashableF32(left.0 * right.0)),
        BinaryIntrinsicFn::Div => ConstValue::F32(HashableF32(if right.0 == 0.0 {
            left.0
        } else {
            left.0 / right.0
        })),
        BinaryIntrinsicFn::Eq => ConstValue::Bool(left.0 == right.0),
        BinaryIntrinsicFn::Ne => ConstValue::Bool(left.0 != right.0),
        BinaryIntrinsicFn::Lt => ConstValue::Bool(left.0 < right.0),
        BinaryIntrinsicFn::Le => ConstValue::Bool(left.0 <= right.0),
        BinaryIntrinsicFn::Gt => ConstValue::Bool(left.0 > right.0),
        BinaryIntrinsicFn::Ge => ConstValue::Bool(left.0 >= right.0),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `f32`"),
    }
}

fn fn_binary_bool_value<'item>(
    left: bool,
    right: bool,
    fn_: BinaryIntrinsicFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryIntrinsicFn::Eq => ConstValue::Bool(left == right),
        BinaryIntrinsicFn::Ne => ConstValue::Bool(left != right),
        BinaryIntrinsicFn::Lt => ConstValue::Bool(!left && right),
        BinaryIntrinsicFn::Le => ConstValue::Bool(left <= right),
        BinaryIntrinsicFn::Gt => ConstValue::Bool(left && !right),
        BinaryIntrinsicFn::Ge => ConstValue::Bool(left >= right),
        BinaryIntrinsicFn::And => ConstValue::Bool(left && right),
        BinaryIntrinsicFn::Or => ConstValue::Bool(left || right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `bool`"),
    }
}

fn fn_binary_typeref_value<'item>(
    left: &'item StructDefinition,
    right: &'item StructDefinition,
    fn_: BinaryIntrinsicFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryIntrinsicFn::Eq => ConstValue::Bool(left == right),
        BinaryIntrinsicFn::Ne => ConstValue::Bool(left != right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `typeref`"),
    }
}

fn fn_unary_value<'item>(
    source: &'item FnDefinition,
    fn_: UnaryIntrinsicFn,
    state: &State<'item>,
) -> ConstValue<'item> {
    match state.const_value(source.params.params[0].id) {
        ConstValue::I32(value) => ConstValue::I32(value.wrapping_neg()),
        ConstValue::F32(value) => ConstValue::F32(HashableF32(-value.0)),
        ConstValue::Bool(value) => ConstValue::Bool(!value),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function"),
    }
}

fn mul_add_value<'item>(source: &'item FnDefinition, state: &State<'item>) -> ConstValue<'item> {
    let (ConstValue::F32(value), ConstValue::F32(multiplier), ConstValue::F32(addend)) = (
        state.const_value(source.params.params[0].id),
        state.const_value(source.params.params[1].id),
        state.const_value(source.params.params[2].id),
    ) else {
        unreachable!("not implemented `{}` constant GPU function", source.name)
    };
    ConstValue::F32(HashableF32(value.0.mul_add(multiplier.0, addend.0)))
}

fn typeof_value<'item>(call: &Call, state: &State<'item>) -> ConstValue<'item> {
    match types::expr_type(&call.args[0].value, state) {
        Type::Struct(type_) => ConstValue::TypeRef(type_),
        Type::Param(param) => ConstValue::Param(param),
        Type::Wildcard(param) => ConstValue::WildcardType(param),
        Type::NoReturn | Type::Unknown => ConstValue::Unknown,
    }
}

fn sizeof_value<'item>(source: &'item FnDefinition, state: &State<'item>) -> ConstValue<'item> {
    match state.const_value(source.params.params[0].id) {
        ConstValue::TypeRef(type_) => ConstValue::U32(type_.size()),
        _ => unreachable!("not implemented `{}` constant GPU function", source.name),
    }
}
