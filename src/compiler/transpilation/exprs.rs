use crate::compiler::consts::ConstValue;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::transpilation::{MAIN_BUFFER_NAME, SpecializedFn, Transpiler};
use crate::utils::{endianness, formatting};
use std::fmt::Write;

impl<'item> Transpiler<'item, '_> {
    pub(crate) fn transpile_expr(&mut self, node: &Expr) {
        let value = self.type_resolver.const_resolver.expr_value(node);
        if value == ConstValue::RuntimeValue {
            match node {
                Expr::Call(child) => self.transpile_call(child),
                Expr::Ident(child) => self.transpile_ident(child),
                Expr::F32Literal(_)
                | Expr::U32Literal(_)
                | Expr::I32Literal(_)
                | Expr::BoolLiteral(_)
                | Expr::Wildcard(_) => unreachable!("expression should be validated before"),
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
            ItemRef::Fn(child) => match &child.body {
                FnBody::Compilerimpl(_) => self.transpile_compilerimpl_fn_call(node),
                FnBody::Statements(body) => self.transpile_custom_fn_call(node, child, body),
            },
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("function calls cannot reference values")
            }
        }
    }

    fn transpile_compilerimpl_fn_call(&mut self, node: &Call) {
        if node.name == "__add__" {
            self.transpile_expr(&node.args[0]);
            self.shader += " + ";
            self.transpile_expr(&node.args[1]);
        } else {
            unreachable!("not implemented `{}` GPU function", node.name);
        }
    }

    fn transpile_custom_fn_call(
        &mut self,
        node: &Call,
        child: &'item FnDefinition,
        body: &'item FnStatementsBody,
    ) {
        let specialized_fn_id = self.register_specialized_fn(node, child, body);
        _ = write!(self.shader, "_{}_{specialized_fn_id}", child.id);
        self.shader += "(";
        for (arg, param) in node.args.iter().zip(&child.params.params) {
            if param.const_mark_span().is_none() {
                self.transpile_expr(arg);
                self.shader += ", ";
            }
        }
        self.shader += ")";
    }

    fn register_specialized_fn(
        &mut self,
        node: &Call,
        child: &'item FnDefinition,
        body: &'item FnStatementsBody,
    ) -> usize {
        let const_param_values = node
            .args
            .iter()
            .zip(&child.params.params)
            .filter(|(_, param)| param.const_mark_span().is_some())
            .map(|(arg, _)| self.type_resolver.const_resolver.expr_value(arg))
            .collect::<Vec<_>>();
        let specialized_fn_id = self.specialized_fns.len();
        *self
            .specialized_fns
            .entry(SpecializedFn {
                fn_: child,
                const_param_values,
                fn_body: body,
            })
            .or_insert(specialized_fn_id)
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
                _ = write!(self.shader, "f32({})", formatting::f32_to_string(value.0));
            }
            ConstValue::Bool(value) => _ = write!(self.shader, "u32({})", u32::from(*value)),
            ConstValue::Param(_) | ConstValue::Unknown | ConstValue::RuntimeValue => {
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
