use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{CompilerImpl, FnDefinition};
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
            Some(
                compilerimpl @ (CompilerImpl::Add
                | CompilerImpl::Sub
                | CompilerImpl::Mul
                | CompilerImpl::Div
                | CompilerImpl::Mod
                | CompilerImpl::Eq
                | CompilerImpl::Ne
                | CompilerImpl::Lt
                | CompilerImpl::Le
                | CompilerImpl::Gt
                | CompilerImpl::Ge
                | CompilerImpl::And
                | CompilerImpl::Or),
            ) => self.fn_compilerimpl_binary_const_value(source, compilerimpl),
            Some(compilerimpl @ (CompilerImpl::Neg | CompilerImpl::Not)) => {
                self.fn_compilerimpl_unary_const_value(source, compilerimpl)
            }
            Some(CompilerImpl::MulAdd) => self.fn_compilerimpl_mul_add_const_value(source),
            Some(CompilerImpl::Typeof) => self.fn_compilerimpl_typeof_const_value(call),
            Some(CompilerImpl::Sizeof) => self.fn_compilerimpl_sizeof_const_value(source),
            None => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    #[allow(
        clippy::bool_comparison,
        clippy::float_cmp,
        clippy::integer_division,
        clippy::wildcard_enum_match_arm
    )]
    fn fn_compilerimpl_binary_const_value(
        &self,
        source: &FnDefinition,
        compilerimpl: CompilerImpl,
    ) -> ConstValue<'item> {
        let left = self.const_value(source.params.params[0].id);
        let right = self.const_value(source.params.params[1].id);
        match (left, right) {
            (ConstValue::I32(left), ConstValue::I32(right)) => match compilerimpl {
                CompilerImpl::Add => ConstValue::I32(left.wrapping_add(right)),
                CompilerImpl::Sub => ConstValue::I32(left.wrapping_sub(right)),
                CompilerImpl::Mul => ConstValue::I32(left.wrapping_mul(right)),
                CompilerImpl::Div => ConstValue::I32(if right == 0 {
                    left
                } else {
                    left.wrapping_div(right)
                }),
                CompilerImpl::Mod => ConstValue::I32(if right == 0 {
                    left
                } else {
                    left.wrapping_rem(right)
                }),
                CompilerImpl::Eq => ConstValue::Bool(left == right),
                CompilerImpl::Ne => ConstValue::Bool(left != right),
                CompilerImpl::Lt => ConstValue::Bool(left < right),
                CompilerImpl::Le => ConstValue::Bool(left <= right),
                CompilerImpl::Gt => ConstValue::Bool(left > right),
                CompilerImpl::Ge => ConstValue::Bool(left >= right),
                _ => unreachable!("invalid i32 compiler implementation"),
            },
            (ConstValue::U32(left), ConstValue::U32(right)) => match compilerimpl {
                CompilerImpl::Add => ConstValue::U32(left.wrapping_add(right)),
                CompilerImpl::Sub => ConstValue::U32(left.wrapping_sub(right)),
                CompilerImpl::Mul => ConstValue::U32(left.wrapping_mul(right)),
                CompilerImpl::Div => ConstValue::U32(if right == 0 { left } else { left / right }),
                CompilerImpl::Mod => ConstValue::U32(if right == 0 { left } else { left % right }),
                CompilerImpl::Eq => ConstValue::Bool(left == right),
                CompilerImpl::Ne => ConstValue::Bool(left != right),
                CompilerImpl::Lt => ConstValue::Bool(left < right),
                CompilerImpl::Le => ConstValue::Bool(left <= right),
                CompilerImpl::Gt => ConstValue::Bool(left > right),
                CompilerImpl::Ge => ConstValue::Bool(left >= right),
                _ => unreachable!("invalid u32 compiler implementation"),
            },
            (ConstValue::F32(left), ConstValue::F32(right)) => match compilerimpl {
                CompilerImpl::Add => ConstValue::F32(HashableF32(left.0 + right.0)),
                CompilerImpl::Sub => ConstValue::F32(HashableF32(left.0 - right.0)),
                CompilerImpl::Mul => ConstValue::F32(HashableF32(left.0 * right.0)),
                CompilerImpl::Div => ConstValue::F32(HashableF32(left.0 / right.0)),
                CompilerImpl::Mod => ConstValue::F32(HashableF32(left.0 % right.0)),
                CompilerImpl::Eq => ConstValue::Bool(left.0 == right.0),
                CompilerImpl::Ne => ConstValue::Bool(left.0 != right.0),
                CompilerImpl::Lt => ConstValue::Bool(left.0 < right.0),
                CompilerImpl::Le => ConstValue::Bool(left.0 <= right.0),
                CompilerImpl::Gt => ConstValue::Bool(left.0 > right.0),
                CompilerImpl::Ge => ConstValue::Bool(left.0 >= right.0),
                _ => unreachable!("invalid f32 compiler implementation"),
            },
            (ConstValue::Bool(left), ConstValue::Bool(right)) => match compilerimpl {
                CompilerImpl::Eq => ConstValue::Bool(left == right),
                CompilerImpl::Ne => ConstValue::Bool(left != right),
                CompilerImpl::Lt => ConstValue::Bool(left < right),
                CompilerImpl::Le => ConstValue::Bool(left <= right),
                CompilerImpl::Gt => ConstValue::Bool(left > right),
                CompilerImpl::Ge => ConstValue::Bool(left >= right),
                CompilerImpl::And => ConstValue::Bool(left && right),
                CompilerImpl::Or => ConstValue::Bool(left || right),
                _ => unreachable!("invalid bool compiler implementation"),
            },
            (ConstValue::TypeRef(left), ConstValue::TypeRef(right)) => match compilerimpl {
                CompilerImpl::Eq => ConstValue::Bool(left == right),
                CompilerImpl::Ne => ConstValue::Bool(left != right),
                _ => unreachable!("invalid typeref compiler implementation"),
            },
            _ => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn fn_compilerimpl_unary_const_value(
        &self,
        source: &FnDefinition,
        compilerimpl: CompilerImpl,
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

    #[allow(clippy::wildcard_enum_match_arm)]
    fn fn_compilerimpl_sizeof_const_value(&self, source: &FnDefinition) -> ConstValue<'item> {
        match self.const_value(source.params.params[0].id) {
            ConstValue::TypeRef(type_) => ConstValue::U32(type_.size()),
            _ => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }
}
