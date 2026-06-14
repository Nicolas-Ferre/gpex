use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{CompilerImpl, FnDefinition};
use crate::compiler::values::ValueResolver;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types::Type;

impl<'item> ValueResolver<'item, '_> {
    pub(super) fn fn_compilerimpl_const_value(
        &mut self,
        call: &Call,
        source: &FnDefinition,
    ) -> ConstValue<'item> {
        match source.compilerimpl() {
            Some(CompilerImpl::Add) => self.fn_compilerimpl_add_const_value(source),
            Some(CompilerImpl::Typeof) => self.fn_compilerimpl_typeof_const_value(call),
            Some(CompilerImpl::Sizeof) => self.fn_compilerimpl_sizeof_const_value(source),
            None => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    fn fn_compilerimpl_add_const_value(&self, source: &FnDefinition) -> ConstValue<'item> {
        let left = self.const_value(source.params.params[0].id);
        let right = self.const_value(source.params.params[1].id);
        match (left, right) {
            (ConstValue::I32(left), ConstValue::I32(right)) => {
                ConstValue::I32(left.wrapping_add(right))
            }
            _ => unreachable!("not implemented `{}` constant GPU function", source.name),
        }
    }

    fn fn_compilerimpl_typeof_const_value(&mut self, call: &Call) -> ConstValue<'item> {
        match self.expr_type(&call.args[0]) {
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
