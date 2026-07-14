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
        if fn_.returns_bool() {
            self.shader += "u32(";
        }
        self.shader += "(";
        self.transpile_compilerimpl_fn_call_binary_operand(&node.args[0].value, fn_.is_bool());
        _ = write!(self.shader, " {} ", fn_.wgsl_operator());
        self.transpile_compilerimpl_fn_call_binary_operand(&node.args[1].value, fn_.is_bool());
        self.shader += ")";
        if fn_.returns_bool() {
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

// TODO: avoid create external impl block (i.e. in other file than item def) for helper functions
impl BinaryCompilerImplFn {
    fn wgsl_operator(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::And => "&&",
            Self::Or => "||",
        }
    }

    fn is_bool(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    fn returns_bool(self) -> bool {
        matches!(
            self,
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge | Self::And | Self::Or
        )
    }
}
