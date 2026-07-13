use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{
    BinaryCompilerImpl, CompilerImpl, FnDefinition, UnaryCompilerImpl,
};
use crate::compiler::values::ValueResolver;
use crate::compiler::values::consts::{ConstValue, HashableF32};
use crate::compiler::values::types::Type;

impl<'item> ValueResolver<'item, '_> {
    pub(super) fn fn_compilerimpl_const_value(
        &mut self,
        call: &Call,
        source: &FnDefinition,
    ) -> ConstValue<'item> {
        match source.compilerimpl() {
            Some(CompilerImpl::Binary(compilerimpl)) => {
                self.fn_compilerimpl_binary_const_value(source, compilerimpl)
            }
            Some(CompilerImpl::Unary(compilerimpl)) => {
                self.fn_compilerimpl_unary_const_value(source, compilerimpl)
            }
            Some(CompilerImpl::MulAdd) => self.fn_compilerimpl_mul_add_const_value(source),
            Some(CompilerImpl::Typeof) => self.fn_compilerimpl_typeof_const_value(call),
            Some(CompilerImpl::Sizeof) => self.fn_compilerimpl_sizeof_const_value(source),
            None => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    // TODO: refactor this function as it is too big
    #[allow(
        clippy::bool_comparison, // needed
        clippy::float_cmp, // needed
        clippy::wildcard_enum_match_arm, // opt-in is preferred
    )]
    fn fn_compilerimpl_binary_const_value(
        &self,
        source: &FnDefinition,
        compilerimpl: BinaryCompilerImpl,
    ) -> ConstValue<'item> {
        let left = self.const_value(source.params.params[0].id);
        let right = self.const_value(source.params.params[1].id);
        match (left, right) {
            (ConstValue::I32(left), ConstValue::I32(right)) => match compilerimpl {
                BinaryCompilerImpl::Add => ConstValue::I32(left.wrapping_add(right)),
                BinaryCompilerImpl::Sub => ConstValue::I32(left.wrapping_sub(right)),
                BinaryCompilerImpl::Mul => ConstValue::I32(left.wrapping_mul(right)),
                BinaryCompilerImpl::Div => ConstValue::I32(if right == 0 {
                    left
                } else {
                    left.wrapping_div(right)
                }),
                BinaryCompilerImpl::Mod => ConstValue::I32(if right == 0 {
                    left
                } else {
                    left.wrapping_rem(right)
                }),
                BinaryCompilerImpl::Eq => ConstValue::Bool(left == right),
                BinaryCompilerImpl::Ne => ConstValue::Bool(left != right),
                BinaryCompilerImpl::Lt => ConstValue::Bool(left < right),
                BinaryCompilerImpl::Le => ConstValue::Bool(left <= right),
                BinaryCompilerImpl::Gt => ConstValue::Bool(left > right),
                BinaryCompilerImpl::Ge => ConstValue::Bool(left >= right),
                _ => unreachable!("invalid i32 compiler implementation"),
            },
            (ConstValue::U32(left), ConstValue::U32(right)) => match compilerimpl {
                BinaryCompilerImpl::Add => ConstValue::U32(left.wrapping_add(right)),
                BinaryCompilerImpl::Sub => ConstValue::U32(left.wrapping_sub(right)),
                BinaryCompilerImpl::Mul => ConstValue::U32(left.wrapping_mul(right)),
                BinaryCompilerImpl::Div => ConstValue::U32(if right == 0 {
                    left
                } else {
                    left.div_euclid(right)
                }),
                BinaryCompilerImpl::Mod => {
                    ConstValue::U32(if right == 0 { left } else { left % right })
                }
                BinaryCompilerImpl::Eq => ConstValue::Bool(left == right),
                BinaryCompilerImpl::Ne => ConstValue::Bool(left != right),
                BinaryCompilerImpl::Lt => ConstValue::Bool(left < right),
                BinaryCompilerImpl::Le => ConstValue::Bool(left <= right),
                BinaryCompilerImpl::Gt => ConstValue::Bool(left > right),
                BinaryCompilerImpl::Ge => ConstValue::Bool(left >= right),
                _ => unreachable!("invalid u32 compiler implementation"),
            },
            (ConstValue::F32(left), ConstValue::F32(right)) => match compilerimpl {
                BinaryCompilerImpl::Add => ConstValue::F32(HashableF32(left.0 + right.0)),
                BinaryCompilerImpl::Sub => ConstValue::F32(HashableF32(left.0 - right.0)),
                BinaryCompilerImpl::Mul => ConstValue::F32(HashableF32(left.0 * right.0)),
                BinaryCompilerImpl::Div => ConstValue::F32(HashableF32(left.0 / right.0)),
                BinaryCompilerImpl::Mod => ConstValue::F32(HashableF32(left.0 % right.0)),
                BinaryCompilerImpl::Eq => ConstValue::Bool(left.0 == right.0),
                BinaryCompilerImpl::Ne => ConstValue::Bool(left.0 != right.0),
                BinaryCompilerImpl::Lt => ConstValue::Bool(left.0 < right.0),
                BinaryCompilerImpl::Le => ConstValue::Bool(left.0 <= right.0),
                BinaryCompilerImpl::Gt => ConstValue::Bool(left.0 > right.0),
                BinaryCompilerImpl::Ge => ConstValue::Bool(left.0 >= right.0),
                _ => unreachable!("invalid f32 compiler implementation"),
            },
            (ConstValue::Bool(left), ConstValue::Bool(right)) => match compilerimpl {
                BinaryCompilerImpl::Eq => ConstValue::Bool(left == right),
                BinaryCompilerImpl::Ne => ConstValue::Bool(left != right),
                BinaryCompilerImpl::Lt => ConstValue::Bool(left < right),
                BinaryCompilerImpl::Le => ConstValue::Bool(left <= right),
                BinaryCompilerImpl::Gt => ConstValue::Bool(left > right),
                BinaryCompilerImpl::Ge => ConstValue::Bool(left >= right),
                BinaryCompilerImpl::And => ConstValue::Bool(left && right),
                BinaryCompilerImpl::Or => ConstValue::Bool(left || right),
                _ => unreachable!("invalid bool compiler implementation"),
            },
            (ConstValue::TypeRef(left), ConstValue::TypeRef(right)) => match compilerimpl {
                BinaryCompilerImpl::Eq => ConstValue::Bool(left == right),
                BinaryCompilerImpl::Ne => ConstValue::Bool(left != right),
                _ => unreachable!("invalid typeref compiler implementation"),
            },
            _ => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    #[expect(clippy::wildcard_enum_match_arm)] // opt-in is preferred
    fn fn_compilerimpl_unary_const_value(
        &self,
        source: &FnDefinition,
        compilerimpl: UnaryCompilerImpl,
    ) -> ConstValue<'item> {
        match self.const_value(source.params.params[0].id) {
            ConstValue::I32(value) => ConstValue::I32(value.wrapping_neg()),
            ConstValue::F32(value) => ConstValue::F32(HashableF32(-value.0)),
            ConstValue::Bool(value) => ConstValue::Bool(!value),
            _ => unreachable!("not implemented `{:?}` constant GPU function", compilerimpl),
        }
    }

    fn fn_compilerimpl_mul_add_const_value(&self, source: &FnDefinition) -> ConstValue<'item> {
        match (
            self.const_value(source.params.params[0].id),
            self.const_value(source.params.params[1].id),
            self.const_value(source.params.params[2].id),
        ) {
            (ConstValue::F32(value), ConstValue::F32(multiplier), ConstValue::F32(addend)) => {
                ConstValue::F32(HashableF32(value.0.mul_add(multiplier.0, addend.0)))
            }
            _ => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    fn fn_compilerimpl_typeof_const_value(&mut self, call: &Call) -> ConstValue<'item> {
        match self.expr_type(&call.args[0].value) {
            Type::Struct(type_) => ConstValue::TypeRef(type_),
            Type::Param(param) => ConstValue::Param(param),
            Type::Wildcard(param) => ConstValue::WildcardType(param),
            Type::NoReturn | Type::Unknown => ConstValue::Unknown,
        }
    }

    #[expect(clippy::wildcard_enum_match_arm)] // opt-in is preferred
    fn fn_compilerimpl_sizeof_const_value(&self, source: &FnDefinition) -> ConstValue<'item> {
        match self.const_value(source.params.params[0].id) {
            ConstValue::TypeRef(type_) => ConstValue::U32(type_.size()),
            _ => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }
}
