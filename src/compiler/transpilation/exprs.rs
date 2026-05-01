use crate::compiler::consts::ConstValue;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::transpilation::{MAIN_BUFFER_NAME, Transpiler};
use crate::utils::{endianness, formatting};
use std::fmt::Write;

impl Transpiler<'_, '_> {
    pub(crate) fn transpile_expr(&mut self, node: &Expr) {
        let value = self.const_checker.expr_value(node);
        if value == ConstValue::RuntimeValue {
            match node {
                Expr::Call(child) => self.transpile_call(child),
                Expr::Ident(child) => self.transpile_ident(child),
                Expr::F32Literal(_)
                | Expr::U32Literal(_)
                | Expr::I32Literal(_)
                | Expr::BoolLiteral(_) => unreachable!("literals should be constant"),
            }
        } else {
            self.transpile_const_value(&value);
        }
    }

    pub(crate) fn transpile_repeat(&mut self, node: &RepeatDefinition) {
        self.transpile_call(&node.call);
        self.shader += "; ";
    }

    pub(crate) fn transpile_var_ref(&mut self, node: &VarDefinition) {
        self.shader += MAIN_BUFFER_NAME;
        _ = write!(self.shader, ".v{}", node.id);
    }

    fn transpile_call(&mut self, node: &Call) {
        match self.indexes.sources[&node.id] {
            ItemRef::Fn(child) => {
                _ = write!(self.shader, "_{}", child.id);
                self.shader += "(";
                for arg in &node.args {
                    self.transpile_expr(arg);
                    self.shader += ", ";
                }
                self.shader += ")";
            }
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("function calls cannot reference values")
            }
        }
    }

    fn transpile_ident(&mut self, node: &Ident) {
        match self.indexes.sources[&node.id] {
            ItemRef::Var(node) => self.transpile_var_ref(node),
            ItemRef::Param(node) => self.transpile_param_ref(node),
            ItemRef::Fn(_) => unreachable!("identifiers cannot reference functions"),
            ItemRef::Const(_) | ItemRef::Struct(_) => {
                unreachable!("constant item should be transpiled in `Expression::transpile`")
            }
        }
    }

    fn transpile_const_value(&mut self, node: &ConstValue<'_>) {
        match node {
            ConstValue::TypeRef(value) => self.transpile_struct_ref(value),
            ConstValue::I32(value) => _ = write!(self.shader, "i32({value})"),
            ConstValue::U32(value) => _ = write!(self.shader, "u32({value})"),
            ConstValue::F32(value) => {
                _ = write!(self.shader, "f32({})", formatting::f32_to_string(*value));
            }
            ConstValue::Bool(value) => _ = write!(self.shader, "u32({})", u32::from(*value)),
            ConstValue::Unknown | ConstValue::RuntimeValue => {
                unreachable!("non-constant cannot be transpiled")
            }
        }
    }

    fn transpile_param_ref(&mut self, node: &Param) {
        let id = node.id;
        _ = write!(self.shader, "_{id}");
    }

    fn transpile_struct_ref(&mut self, node: &StructDefinition) {
        let [id_part1, id_part2] = endianness::to_portable_u32x2(node.id);
        _ = write!(self.shader, "vec2<u32>({id_part1}, {id_part2})");
    }
}
