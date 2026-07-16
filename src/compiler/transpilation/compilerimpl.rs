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
            Some(CompilerImplFn::Binary(fn_)) => {
                self.transpile_fn_call_binary(node, fn_);
            }
            Some(CompilerImplFn::Unary(fn_)) => {
                self.transpile_fn_call_unary(node, fn_);
            }
            Some(CompilerImplFn::MulAdd) => self.transpile_mul_add(node),
            Some(CompilerImplFn::Typeof | CompilerImplFn::Sizeof) | None => {
                unreachable!("not implemented `{}` GPU function", source.name)
            }
        }
    }

    fn transpile_fn_call_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        match self.type_(&node.args[0].value) {
            CompilerImplType::I32 | CompilerImplType::U32 => {
                self.transpile_fn_call_int_binary(node, fn_);
            }
            CompilerImplType::F32 => self.transpile_fn_call_f32_binary(node, fn_),
            CompilerImplType::Bool => self.transpile_fn_call_bool_binary(node, fn_),
            CompilerImplType::Typeref => self.transpile_fn_call_typeref_binary(node, fn_),
        }
    }

    fn transpile_fn_call_int_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        let is_divisor_zero = self.is_zero_int(&node.args[1].value);
        if is_divisor_zero && fn_ == BinaryCompilerImplFn::Div {
            self.transpile_arg(&node.args[0], true);
        } else if is_divisor_zero && fn_ == BinaryCompilerImplFn::Mod {
            self.transpile_arg(&node.args[1], true);
        } else {
            self.transpile_fn_call_scalar_binary(node, fn_, true);
        }
    }

    fn transpile_fn_call_f32_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        if fn_ == BinaryCompilerImplFn::Div {
            self.shader += "(";
            self.transpile_arg(&node.args[0], true);
            self.shader += " / select(";
            self.transpile_arg(&node.args[1], true);
            self.shader += ", f32(1), ";
            self.transpile_arg(&node.args[1], true);
            self.shader += " == f32(0)))";
        } else {
            self.transpile_fn_call_scalar_binary(node, fn_, true);
        }
    }

    fn transpile_fn_call_bool_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        let is_bool_arg_converted = !fn_.is_comparison_operator();
        self.transpile_fn_call_scalar_binary(node, fn_, is_bool_arg_converted);
    }

    fn transpile_fn_call_typeref_binary(&mut self, node: &Call, fn_: BinaryCompilerImplFn) {
        let negation = wgsl_comparison_to_negation_operator(fn_);
        _ = write!(self.shader, "u32({negation}all(");
        self.transpile_arg(&node.args[0], true);
        self.shader += " == ";
        self.transpile_arg(&node.args[1], true);
        self.shader += "))";
    }

    fn transpile_fn_call_scalar_binary(
        &mut self,
        node: &Call,
        fn_: BinaryCompilerImplFn,
        is_bool_arg_converted: bool,
    ) {
        if fn_.is_comparison_operator() || fn_.is_logical_operator() {
            self.shader += "u32(";
        }
        self.shader += "(";
        self.transpile_arg(&node.args[0], is_bool_arg_converted);
        _ = write!(self.shader, " {} ", wgsl_binary_operator(fn_));
        self.transpile_arg(&node.args[1], is_bool_arg_converted);
        self.shader += ")";
        if fn_.is_comparison_operator() || fn_.is_logical_operator() {
            self.shader += ")";
        }
    }

    fn transpile_fn_call_unary(&mut self, node: &Call, fn_: UnaryCompilerImplFn) {
        let (prefix, suffix) = match fn_ {
            UnaryCompilerImplFn::Neg => ("(-", ")"),
            UnaryCompilerImplFn::Not => ("u32(!", ")"),
        };
        self.shader += prefix;
        self.transpile_arg(&node.args[0], true);
        self.shader += suffix;
    }

    fn transpile_mul_add(&mut self, node: &Call) {
        self.shader += "fma(";
        for arg in &node.args {
            self.transpile_arg(arg, true);
            self.shader += ", ";
        }
        self.shader += ")";
    }

    fn transpile_arg(&mut self, arg: &Arg, is_bool_converted: bool) {
        let arg_type = self.type_(&arg.value);
        if arg_type == CompilerImplType::Bool && is_bool_converted {
            self.shader += "(";
        }
        self.transpile_expr(&arg.value);
        if arg_type == CompilerImplType::Bool && is_bool_converted {
            self.shader += " == u32(true))";
        }
    }

    fn is_zero_int(&mut self, node: &Expr) -> bool {
        let value = self.value_resolver.expr_const_value(node);
        matches!(value, ConstValue::I32(0) | ConstValue::U32(0))
    }

    fn type_(&mut self, node: &Expr) -> CompilerImplType {
        let arg_type = self.value_resolver.expr_type(node);
        if arg_type == Type::Struct(self.indexes.search_prelude_type("f32")) {
            CompilerImplType::F32
        } else if arg_type == Type::Struct(self.indexes.search_prelude_type("i32")) {
            CompilerImplType::I32
        } else if arg_type == Type::Struct(self.indexes.search_prelude_type("u32")) {
            CompilerImplType::U32
        } else if arg_type == Type::Struct(self.indexes.search_prelude_type("bool")) {
            CompilerImplType::Bool
        } else if arg_type == Type::Struct(self.indexes.search_prelude_type("typeref")) {
            CompilerImplType::Typeref
        } else {
            unreachable!("unsupported `compilerimpl` type")
        }
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

#[expect(clippy::wildcard_enum_match_arm)]
fn wgsl_comparison_to_negation_operator(fn_: BinaryCompilerImplFn) -> &'static str {
    match fn_ {
        BinaryCompilerImplFn::Eq => "",
        BinaryCompilerImplFn::Ne => "!",
        _ => unreachable!("invalid typeref compiler implementation"),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CompilerImplType {
    I32,
    U32,
    F32,
    Bool,
    Typeref,
}
