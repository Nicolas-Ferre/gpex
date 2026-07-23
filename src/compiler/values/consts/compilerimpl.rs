#![expect(clippy::wildcard_enum_match_arm)] // opt-in is preferred

use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{
    BinaryCompilerImplFn, CompilerImplFn, FnDefinition, UnaryCompilerImplFn,
};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::state::State;
use crate::compiler::values::consts::{ConstValue, HashableF32};
use crate::compiler::values::types;
use crate::compiler::values::types::Type;

pub(super) fn call_value<'item>(
    call: &Call,
    source: &'item FnDefinition,
    state: &mut State<'item>,
) -> ConstValue<'item> {
    for param in &source.params.params {
        if matches!(
            state.const_value(param.id),
            ConstValue::Param(_) | ConstValue::WildcardType(_)
        ) {
            return ConstValue::Unknown;
        }
    }
    match source.compilerimpl() {
        Some(CompilerImplFn::Binary(fn_)) => fn_binary_value(source, fn_, state),
        Some(CompilerImplFn::Unary(fn_)) => fn_unary_value(source, fn_, state),
        Some(CompilerImplFn::MulAdd) => mul_add_value(source, state),
        Some(CompilerImplFn::Sizeof) => sizeof_value(source, state),
        Some(CompilerImplFn::Typeof) => typeof_value(call, state),
        None => unreachable!("not implemented `{}` constant GPU function", source.name),
    }
}

fn fn_binary_value<'item>(
    source: &'item FnDefinition,
    fn_: BinaryCompilerImplFn,
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

fn fn_binary_i32_value<'item>(
    left: i32,
    right: i32,
    fn_: BinaryCompilerImplFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryCompilerImplFn::Add => ConstValue::I32(left.wrapping_add(right)),
        BinaryCompilerImplFn::Sub => ConstValue::I32(left.wrapping_sub(right)),
        BinaryCompilerImplFn::Mul => ConstValue::I32(left.wrapping_mul(right)),
        BinaryCompilerImplFn::Div => ConstValue::I32(if right == 0 {
            left
        } else {
            left.wrapping_div(right)
        }),
        BinaryCompilerImplFn::Mod => ConstValue::I32(if right == 0 {
            0
        } else {
            left.wrapping_rem(right)
        }),
        BinaryCompilerImplFn::Eq => ConstValue::Bool(left == right),
        BinaryCompilerImplFn::Ne => ConstValue::Bool(left != right),
        BinaryCompilerImplFn::Lt => ConstValue::Bool(left < right),
        BinaryCompilerImplFn::Le => ConstValue::Bool(left <= right),
        BinaryCompilerImplFn::Gt => ConstValue::Bool(left > right),
        BinaryCompilerImplFn::Ge => ConstValue::Bool(left >= right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `i32`"),
    }
}

fn fn_binary_u32_value<'item>(
    left: u32,
    right: u32,
    fn_: BinaryCompilerImplFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryCompilerImplFn::Add => ConstValue::U32(left.wrapping_add(right)),
        BinaryCompilerImplFn::Sub => ConstValue::U32(left.wrapping_sub(right)),
        BinaryCompilerImplFn::Mul => ConstValue::U32(left.wrapping_mul(right)),
        BinaryCompilerImplFn::Div => ConstValue::U32(if right == 0 {
            left
        } else {
            left.wrapping_div(right)
        }),
        BinaryCompilerImplFn::Mod => ConstValue::U32(if right == 0 {
            0
        } else {
            left.wrapping_rem(right)
        }),
        BinaryCompilerImplFn::Eq => ConstValue::Bool(left == right),
        BinaryCompilerImplFn::Ne => ConstValue::Bool(left != right),
        BinaryCompilerImplFn::Lt => ConstValue::Bool(left < right),
        BinaryCompilerImplFn::Le => ConstValue::Bool(left <= right),
        BinaryCompilerImplFn::Gt => ConstValue::Bool(left > right),
        BinaryCompilerImplFn::Ge => ConstValue::Bool(left >= right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `u32`"),
    }
}

#[expect(clippy::float_cmp)] // needed
fn fn_binary_f32_value<'item>(
    left: HashableF32,
    right: HashableF32,
    fn_: BinaryCompilerImplFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryCompilerImplFn::Add => ConstValue::F32(HashableF32(left.0 + right.0)),
        BinaryCompilerImplFn::Sub => ConstValue::F32(HashableF32(left.0 - right.0)),
        BinaryCompilerImplFn::Mul => ConstValue::F32(HashableF32(left.0 * right.0)),
        BinaryCompilerImplFn::Div => ConstValue::F32(HashableF32(if right.0 == 0.0 {
            left.0
        } else {
            left.0 / right.0
        })),
        BinaryCompilerImplFn::Eq => ConstValue::Bool(left.0 == right.0),
        BinaryCompilerImplFn::Ne => ConstValue::Bool(left.0 != right.0),
        BinaryCompilerImplFn::Lt => ConstValue::Bool(left.0 < right.0),
        BinaryCompilerImplFn::Le => ConstValue::Bool(left.0 <= right.0),
        BinaryCompilerImplFn::Gt => ConstValue::Bool(left.0 > right.0),
        BinaryCompilerImplFn::Ge => ConstValue::Bool(left.0 >= right.0),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `f32`"),
    }
}

fn fn_binary_bool_value<'item>(
    left: bool,
    right: bool,
    fn_: BinaryCompilerImplFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryCompilerImplFn::Eq => ConstValue::Bool(left == right),
        BinaryCompilerImplFn::Ne => ConstValue::Bool(left != right),
        BinaryCompilerImplFn::Lt => ConstValue::Bool(!left && right),
        BinaryCompilerImplFn::Le => ConstValue::Bool(left <= right),
        BinaryCompilerImplFn::Gt => ConstValue::Bool(left && !right),
        BinaryCompilerImplFn::Ge => ConstValue::Bool(left >= right),
        BinaryCompilerImplFn::And => ConstValue::Bool(left && right),
        BinaryCompilerImplFn::Or => ConstValue::Bool(left || right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `bool`"),
    }
}

fn fn_binary_typeref_value<'item>(
    left: &'item StructDefinition,
    right: &'item StructDefinition,
    fn_: BinaryCompilerImplFn,
) -> ConstValue<'item> {
    match fn_ {
        BinaryCompilerImplFn::Eq => ConstValue::Bool(left == right),
        BinaryCompilerImplFn::Ne => ConstValue::Bool(left != right),
        _ => unreachable!("not implemented `{fn_:?}` constant GPU function for `typeref`"),
    }
}

fn fn_unary_value<'item>(
    source: &'item FnDefinition,
    fn_: UnaryCompilerImplFn,
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

fn typeof_value<'item>(call: &Call, state: &mut State<'item>) -> ConstValue<'item> {
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
