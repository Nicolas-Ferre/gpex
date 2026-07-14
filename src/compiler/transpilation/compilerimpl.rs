use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{
    BinaryCompilerImplFn, CompilerImplFn, FnDefinition, UnaryCompilerImplFn,
};
use crate::compiler::transpilation::Transpiler;
use crate::compiler::values::types::Type;
use std::fmt::Write;

impl Transpiler<'_, '_> {
    pub(super) fn transpile_compilerimpl_fn_call(&mut self, node: &Call, source: &FnDefinition) {
        match source.compilerimpl() {
            Some(CompilerImplFn::Binary(compilerimpl)) => {
                self.transpile_compilerimpl_fn_call_binary(node, compilerimpl);
            }
            Some(CompilerImplFn::Unary(compilerimpl)) => {
                self.transpile_compilerimpl_fn_call_unary(node, compilerimpl);
            }
            Some(CompilerImplFn::MulAdd) => self.transpile_compilerimpl_fn_call_mul_add(node),
            Some(CompilerImplFn::Typeof | CompilerImplFn::Sizeof) | None => {
                unreachable!("not implemented `{}` GPU function", source.name)
            }
        }
    }

    fn transpile_compilerimpl_fn_call_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        let is_typeref_comparison = matches!(
            self.value_resolver.expr_type(&node.args[0].value),
            Type::Struct(type_) if type_.name == "typeref"
        );
        if is_typeref_comparison {
            self.transpile_compilerimpl_fn_call_typeref_binary(node, fn_);
        } else {
            self.transpile_compilerimpl_fn_call_scalar_binary(node, fn_);
        }
    }

    #[expect(clippy::wildcard_enum_match_arm)] // opt-in is preferred
    fn transpile_compilerimpl_fn_call_typeref_binary(
        &mut self,
        node: &Call,
        fn_: BinaryCompilerImplFn,
    ) {
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

    fn transpile_compilerimpl_fn_call_scalar_binary(
        &mut self,
        node: &Call,
        fn_: BinaryCompilerImplFn,
    ) {
        if is_binary_operator_returning_bool(fn_) {
            self.shader += "u32(";
        }
        self.shader += "(";
        self.transpile_compilerimpl_fn_call_binary_operand(
            &node.args[0].value,
            is_binary_operator_accepting_bool(fn_),
        );
        _ = write!(self.shader, " {} ", binary_operator_wgsl(fn_));
        self.transpile_compilerimpl_fn_call_binary_operand(
            &node.args[1].value,
            is_binary_operator_accepting_bool(fn_),
        );
        self.shader += ")";
        if is_binary_operator_returning_bool(fn_) {
            self.shader += ")";
        }
    }

    fn transpile_compilerimpl_fn_call_binary_operand(&mut self, expr: &Expr, is_bool: bool) {
        if is_bool {
            self.shader += "(";
        }
        self.transpile_expr(expr);
        if is_bool {
            self.shader += " == u32(true))";
        }
    }

    fn transpile_compilerimpl_fn_call_unary(&mut self, node: &Call, fn_: UnaryCompilerImplFn) {
        let (prefix, suffix) = match fn_ {
            UnaryCompilerImplFn::Neg => ("(-", ")"),
            UnaryCompilerImplFn::Not => ("u32(", " == u32(false))"),
        };
        self.shader += prefix;
        self.transpile_expr(&node.args[0].value);
        self.shader += suffix;
    }

    fn transpile_compilerimpl_fn_call_mul_add(&mut self, node: &Call) {
        self.shader += "fma(";
        for arg in &node.args {
            self.transpile_expr(&arg.value);
            self.shader += ", ";
        }
        self.shader += ")";
    }
}

fn binary_operator_wgsl(fn_: BinaryCompilerImplFn) -> &'static str {
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

fn is_binary_operator_accepting_bool(fn_: BinaryCompilerImplFn) -> bool {
    matches!(fn_, BinaryCompilerImplFn::And | BinaryCompilerImplFn::Or)
}

fn is_binary_operator_returning_bool(fn_: BinaryCompilerImplFn) -> bool {
    matches!(
        fn_,
        BinaryCompilerImplFn::Eq
            | BinaryCompilerImplFn::Ne
            | BinaryCompilerImplFn::Lt
            | BinaryCompilerImplFn::Le
            | BinaryCompilerImplFn::Gt
            | BinaryCompilerImplFn::Ge
            | BinaryCompilerImplFn::And
            | BinaryCompilerImplFn::Or
    )
}
