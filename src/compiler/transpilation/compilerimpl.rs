use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{
    BinaryCompilerImplFn, CompilerImplFn, FnDefinition, UnaryCompilerImplFn,
};
use crate::compiler::transpilation::Transpiler;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types::Type;
use std::fmt::Write;

impl Transpiler<'_, '_> {
    pub(super) fn transpile_compilerimpl_fn_call(&mut self, node: &Call, source: &FnDefinition) {
        match source.compilerimpl() {
            Some(CompilerImplFn::Binary(compilerimpl)) => {
                self.transpile_fn_call_binary(node, compilerimpl);
            }
            Some(CompilerImplFn::Unary(compilerimpl)) => {
                self.transpile_fn_call_unary(node, compilerimpl);
            }
            Some(CompilerImplFn::MulAdd) => self.transpile_mul_add(node),
            Some(CompilerImplFn::Typeof | CompilerImplFn::Sizeof) | None => {
                unreachable!("not implemented `{}` GPU function", source.name)
            }
        }
    }

    fn transpile_fn_call_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        let typeref_type = self.indexes.search_prelude_type("typeref");
        let is_typeref_comparison = matches!(
            self.value_resolver.expr_type(&node.args[0].value),
            Type::Struct(type_) if type_ == typeref_type
        );
        if is_typeref_comparison {
            self.transpile_fn_call_typeref_binary(node, fn_);
        } else {
            self.transpile_fn_call_scalar_binary(node, fn_);
        }
    }

    #[expect(clippy::wildcard_enum_match_arm)] // opt-in is preferred
    fn transpile_fn_call_typeref_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        let negation = match fn_ {
            BinaryCompilerImplFn::Eq => "",
            BinaryCompilerImplFn::Ne => "!",
            _ => unreachable!("invalid typeref compiler implementation"),
        };
        _ = write!(self.shader, "u32({negation}all(");
        self.transpile_expr(&node.args[0].value);
        self.shader += " == ";
        self.transpile_expr(&node.args[1].value);
        self.shader += "))";
    }

    fn transpile_fn_call_scalar_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        if let Some(result) = self.int_fn_call_zero_divisor_result(node, fn_) {
            self.transpile_expr(result);
            return;
        }
        let f32_type = self.indexes.search_prelude_type("f32");
        let is_f32_division = fn_ == BinaryCompilerImplFn::Div
            && matches!(
                self.value_resolver.expr_type(&node.args[0].value),
                Type::Struct(type_) if type_ == f32_type
            );
        if is_f32_division {
            self.transpile_fn_call_f32_div(node);
            return;
        }
        self.transpile_fn_call_scalar_binary_operator(node, fn_);
    }

    fn transpile_fn_call_scalar_binary_operator(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        if fn_.is_comparison_operator() || fn_.is_logical_operator() {
            self.shader += "u32(";
        }
        self.shader += "(";
        self.transpile_fn_call_binary_operand(&node.args[0].value, fn_.is_logical_operator());
        _ = write!(self.shader, " {} ", wgsl_binary_operator(fn_));
        self.transpile_fn_call_binary_operand(&node.args[1].value, fn_.is_logical_operator());
        self.shader += ")";
        if fn_.is_comparison_operator() || fn_.is_logical_operator() {
            self.shader += ")";
        }
    }

    fn transpile_fn_call_f32_div(&mut self, node: &Call) {
        self.shader += "(";
        self.transpile_expr(&node.args[0].value);
        self.shader += " / select(";
        self.transpile_expr(&node.args[1].value);
        self.shader += ", f32(1), ";
        self.transpile_expr(&node.args[1].value);
        self.shader += " == f32(0)))";
    }

    fn transpile_fn_call_binary_operand(&mut self, expr: &Expr, is_bool: bool) {
        if is_bool {
            self.shader += "(";
        }
        self.transpile_expr(expr);
        if is_bool {
            self.shader += " == u32(true))";
        }
    }

    fn transpile_fn_call_unary(&mut self, node: &Call, fn_: UnaryCompilerImplFn) {
        let (prefix, suffix) = match fn_ {
            UnaryCompilerImplFn::Neg => ("(-", ")"),
            UnaryCompilerImplFn::Not => ("u32(", " == u32(false))"),
        };
        self.shader += prefix;
        self.transpile_expr(&node.args[0].value);
        self.shader += suffix;
    }

    fn transpile_mul_add(&mut self, node: &Call) {
        self.shader += "fma(";
        for arg in &node.args {
            self.transpile_expr(&arg.value);
            self.shader += ", ";
        }
        self.shader += ")";
    }

    fn int_fn_call_zero_divisor_result<'node>(
        &mut self,
        node: &'node Call,
        fn_: BinaryCompilerImplFn,
    ) -> Option<&'node Expr> {
        let is_divisor_zero = self.is_arg_zero_int(&node.args[1]);
        if is_divisor_zero && fn_ == BinaryCompilerImplFn::Div {
            Some(&node.args[0].value)
        } else if is_divisor_zero && fn_ == BinaryCompilerImplFn::Mod {
            Some(&node.args[1].value)
        } else {
            None
        }
    }

    fn is_arg_zero_int(&mut self, arg: &Arg) -> bool {
        let value = self.value_resolver.expr_const_value(&arg.value);
        matches!(value, ConstValue::I32(0) | ConstValue::U32(0))
    }
}

fn wgsl_binary_operator(fn_: BinaryCompilerImplFn) -> &'static str {
    match fn_ {
        BinaryCompilerImplFn::Add => "+",
        BinaryCompilerImplFn::Sub => "-",
        BinaryCompilerImplFn::Mul => "*",
        BinaryCompilerImplFn::Div => "/",
        BinaryCompilerImplFn::Mod => "%",
        BinaryCompilerImplFn::Eq => "==",
        BinaryCompilerImplFn::Ne => "!=",
        BinaryCompilerImplFn::Lt => "<",
        BinaryCompilerImplFn::Le => "<=",
        BinaryCompilerImplFn::Gt => ">",
        BinaryCompilerImplFn::Ge => ">=",
        BinaryCompilerImplFn::And => "&&",
        BinaryCompilerImplFn::Or => "||",
    }
}
