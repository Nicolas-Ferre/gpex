use crate::compiler::consts::ConstValue;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, ReturnStatement, Statement};
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::transpilation::exprs;
use crate::compiler::transpilation::{SpecializedFn, TranspileState};
use crate::compiler::types;
use crate::compiler::types::Type;
use std::fmt::Write;

pub(super) fn transpile_specialized_fn<'item>(
    fn_: SpecializedFn<'item>,
    fn_index: usize,
    state: &mut TranspileState<'_, 'item>,
) {
    if !state.transpiled_specialized_fn_indexes.insert(fn_index) {
        return;
    }
    state.inner.enter_scope();
    let source = fn_.fn_;
    let id = source.id;
    _ = write!(state.shader, "fn _{id}_{fn_index}");
    transpile_params(
        &source.params,
        fn_.const_param_values.into_iter(),
        fn_.wildcard_param_types.into_iter(),
        state,
    );
    if let Some(return_type) = types::fn_type(source, state.inner).struct_ref() {
        let return_type_name = transpile_type_name(return_type);
        _ = write!(state.shader, " -> {return_type_name} {{ ");
    } else {
        _ = write!(state.shader, " {{ ");
    }
    transpile_mut_param_definitions(&source.params, state);
    for statement in &fn_.fn_body.statements {
        transpile_statement(statement, state);
    }
    _ = write!(state.shader, " }}");
    state.inner.exit_scope();
}

pub(super) fn transpile_var_init(var: &VarDefinition, state: &mut TranspileState<'_, '_>) {
    exprs::transpile_var_ref(var, state);
    state.shader += " = ";
    exprs::transpile_expr(&var.default_value, state);
    state.shader += "; ";
}

pub(super) fn transpile_repeat(repeat: &RepeatDefinition, state: &mut TranspileState<'_, '_>) {
    exprs::transpile_call(&repeat.call, state);
    state.shader += "; ";
}

pub(super) fn transpile_var_as_struct_field(
    var: &VarDefinition,
    state: &mut TranspileState<'_, '_>,
) {
    let type_ = types::var_type(var, state.inner)
        .struct_ref()
        .unwrap_or_else(|| unreachable!("variable type should be validated before"));
    _ = write!(
        state.shader,
        "v{}: {}, ",
        var.id,
        transpile_type_name(type_)
    );
}

fn transpile_statement(statement: &Statement, state: &mut TranspileState<'_, '_>) {
    match statement {
        Statement::Return(return_statement) => transpile_return_statement(return_statement, state),
        Statement::Assignment(assignment) => transpile_assignment_statement(assignment, state),
    }
}

fn transpile_return_statement(return_: &ReturnStatement, state: &mut TranspileState<'_, '_>) {
    state.shader += "return ";
    exprs::transpile_expr(&return_.value, state);
    state.shader += ";";
}

fn transpile_assignment_statement(
    assignment: &AssignmentStatement,
    state: &mut TranspileState<'_, '_>,
) {
    exprs::transpile_expr(&assignment.assigned, state);
    state.shader += " = ";
    exprs::transpile_expr(&assignment.value, state);
    state.shader += ";";
}

fn transpile_params<'item>(
    params: &'item ParamGroup,
    mut const_param_values: impl Iterator<Item = ConstValue<'item>>,
    mut wildcard_param_types: impl Iterator<Item = &'item StructDefinition>,
    state: &mut TranspileState<'_, 'item>,
) {
    state.shader += "(";
    for param in &params.params {
        resolve_param_wildcard_type(param, &mut wildcard_param_types, state);
        if param.const_mark_span().is_some() {
            resolve_const_param_value(param, &mut const_param_values, state);
        } else {
            transpile_param(param, state);
            state.shader += ", ";
        }
    }
    state.shader += ")";
}

fn transpile_param<'item>(param: &'item Param, state: &mut TranspileState<'_, 'item>) {
    let id = param.id;
    let type_ = types::param_type(param, state.inner)
        .struct_ref()
        .unwrap_or_else(|| unreachable!("parameter type should be validated before"));
    let type_name = transpile_type_name(type_);
    _ = write!(state.shader, "_{id}_const: {type_name}");
}

fn transpile_mut_param_definitions(params: &ParamGroup, state: &mut TranspileState<'_, '_>) {
    for param in &params.params {
        if param.const_mark_span().is_none() {
            transpile_mut_param_definition(param, state);
        }
    }
}

fn transpile_mut_param_definition(param: &Param, state: &mut TranspileState<'_, '_>) {
    let id = param.id;
    _ = write!(state.shader, "var _{id} = _{id}_const; ");
}

fn transpile_type_name(type_: &StructDefinition) -> &str {
    match (type_.name_span.file_index, type_.name.as_str()) {
        (PRELUDE_FILE_INDEX, "typeref") => "vec2<u32>",
        (PRELUDE_FILE_INDEX, "f32") => "f32",
        (PRELUDE_FILE_INDEX, "i32") => "i32",
        (PRELUDE_FILE_INDEX, "u32" | "bool") => "u32",
        _ => unreachable!("not implemented `{}` GPU type", type_.name),
    }
}

fn resolve_const_param_value<'item>(
    param: &Param,
    const_param_values: &mut impl Iterator<Item = ConstValue<'item>>,
    state: &TranspileState<'_, 'item>,
) {
    let value = const_param_values
        .next()
        .unwrap_or_else(|| unreachable!("mismatching number of const params"));
    state.inner.add_const_value(param.id, value);
}

fn resolve_param_wildcard_type<'item>(
    param: &Param,
    wildcard_param_types: &mut impl Iterator<Item = &'item StructDefinition>,
    state: &TranspileState<'_, 'item>,
) {
    if !matches!(param.type_, Expr::Wildcard(_)) {
        return;
    }
    let type_ = wildcard_param_types
        .next()
        .unwrap_or_else(|| unreachable!("mismatching number of wildcard params"));
    state.inner.add_wildcard_type(param.id, Type::Struct(type_));
}
