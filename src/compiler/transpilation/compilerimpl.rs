use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::{Arg, Call};
use crate::compiler::parsing::items::fns::{
    BinaryCompilerImplFn, CompilerImplFn, FnDefinition, UnaryCompilerImplFn,
};
use crate::compiler::state::CompilerImplType;
use crate::compiler::transpilation::{TranspileState, exprs};
use crate::compiler::values::consts;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types;
use std::fmt::Write;

pub(super) fn transpile_call(
    call: &Call,
    source: &FnDefinition,
    state: &mut TranspileState<'_, '_>,
) {
    match source.compilerimpl() {
        Some(CompilerImplFn::Binary(fn_)) => {
            transpile_fn_call_binary(call, fn_, state);
        }
        Some(CompilerImplFn::Unary(fn_)) => {
            transpile_fn_call_unary(call, fn_, state);
        }
        Some(CompilerImplFn::MulAdd) => transpile_mul_add(call, state),
        Some(CompilerImplFn::Typeof | CompilerImplFn::Sizeof) | None => {
            unreachable!("not implemented `{}` GPU function", source.name)
        }
    }
}

fn transpile_fn_call_binary(
    call: &Call,
    fn_: BinaryCompilerImplFn,
    state: &mut TranspileState<'_, '_>,
) {
    let type_ = type_(&call.args[0].value, state);
    match type_ {
        CompilerImplType::I32 | CompilerImplType::U32 => {
            transpile_fn_call_int_binary(call, fn_, type_, state);
        }
        CompilerImplType::F32 => transpile_fn_call_f32_binary(call, fn_, type_, state),
        CompilerImplType::Bool => transpile_fn_call_bool_binary(call, fn_, type_, state),
        CompilerImplType::Typeref => transpile_fn_call_typeref_binary(call, fn_, type_, state),
    }
}

fn transpile_fn_call_int_binary(
    call: &Call,
    fn_: BinaryCompilerImplFn,
    type_: CompilerImplType,
    state: &mut TranspileState<'_, '_>,
) {
    let is_divisor_zero = is_zero_int(&call.args[1].value, state);
    if is_divisor_zero && fn_ == BinaryCompilerImplFn::Div {
        transpile_arg(&call.args[0], type_, true, state);
    } else if is_divisor_zero && fn_ == BinaryCompilerImplFn::Mod {
        transpile_arg(&call.args[1], type_, true, state);
    } else {
        transpile_fn_call_scalar_binary(call, fn_, type_, true, state);
    }
}

fn transpile_fn_call_f32_binary(
    call: &Call,
    fn_: BinaryCompilerImplFn,
    type_: CompilerImplType,
    state: &mut TranspileState<'_, '_>,
) {
    if fn_ == BinaryCompilerImplFn::Div {
        state.shader += "(";
        transpile_arg(&call.args[0], type_, true, state);
        state.shader += " / select(";
        transpile_arg(&call.args[1], type_, true, state);
        state.shader += ", f32(1), ";
        transpile_arg(&call.args[1], type_, true, state);
        state.shader += " == f32(0)))";
    } else {
        transpile_fn_call_scalar_binary(call, fn_, type_, true, state);
    }
}

fn transpile_fn_call_bool_binary(
    call: &Call,
    fn_: BinaryCompilerImplFn,
    type_: CompilerImplType,
    state: &mut TranspileState<'_, '_>,
) {
    let is_bool_arg_converted = !fn_.is_comparison_operator();
    transpile_fn_call_scalar_binary(call, fn_, type_, is_bool_arg_converted, state);
}

fn transpile_fn_call_typeref_binary(
    call: &Call,
    fn_: BinaryCompilerImplFn,
    type_: CompilerImplType,
    state: &mut TranspileState<'_, '_>,
) {
    let negation = wgsl_comparison_to_negation_operator(fn_);
    _ = write!(state.shader, "u32({negation}all(");
    transpile_arg(&call.args[0], type_, true, state);
    state.shader += " == ";
    transpile_arg(&call.args[1], type_, true, state);
    state.shader += "))";
}

fn transpile_fn_call_scalar_binary(
    call: &Call,
    fn_: BinaryCompilerImplFn,
    type_: CompilerImplType,
    is_bool_arg_converted: bool,
    state: &mut TranspileState<'_, '_>,
) {
    if fn_.is_comparison_operator() || fn_.is_logical_operator() {
        state.shader += "u32(";
    }
    state.shader += "(";
    transpile_arg(&call.args[0], type_, is_bool_arg_converted, state);
    _ = write!(state.shader, " {} ", wgsl_binary_operator(fn_));
    transpile_arg(&call.args[1], type_, is_bool_arg_converted, state);
    state.shader += ")";
    if fn_.is_comparison_operator() || fn_.is_logical_operator() {
        state.shader += ")";
    }
}

fn transpile_fn_call_unary(
    call: &Call,
    fn_: UnaryCompilerImplFn,
    state: &mut TranspileState<'_, '_>,
) {
    let type_ = type_(&call.args[0].value, state);
    let (prefix, suffix) = match fn_ {
        UnaryCompilerImplFn::Neg => ("(-", ")"),
        UnaryCompilerImplFn::Not => ("u32(!", ")"),
    };
    state.shader += prefix;
    transpile_arg(&call.args[0], type_, true, state);
    state.shader += suffix;
}

fn transpile_mul_add(call: &Call, state: &mut TranspileState<'_, '_>) {
    let type_ = type_(&call.args[0].value, state);
    state.shader += "fma(";
    for arg in &call.args {
        transpile_arg(arg, type_, true, state);
        state.shader += ", ";
    }
    state.shader += ")";
}

fn transpile_arg(
    arg: &Arg,
    type_: CompilerImplType,
    is_bool_converted: bool,
    state: &mut TranspileState<'_, '_>,
) {
    if type_ == CompilerImplType::Bool && is_bool_converted {
        state.shader += "(";
    }
    exprs::transpile_expr(&arg.value, state);
    if type_ == CompilerImplType::Bool && is_bool_converted {
        state.shader += " == u32(true))";
    }
}

fn is_zero_int(expr: &Expr, state: &TranspileState<'_, '_>) -> bool {
    let value = consts::expr_value(expr, state.inner);
    matches!(value, ConstValue::I32(0) | ConstValue::U32(0))
}

fn type_(expr: &Expr, state: &TranspileState<'_, '_>) -> CompilerImplType {
    let type_ = types::expr_type(expr, state.inner)
        .struct_ref()
        .unwrap_or_else(|| unreachable!("unexpected value that is not a type"));
    state
        .inner
        .compilerimpl_type(type_)
        .unwrap_or_else(|| unreachable!("unsupported `compilerimpl` type"))
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
