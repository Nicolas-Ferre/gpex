use crate::BufferField;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::transpilation::TranspileState;
use crate::compiler::types;
use crate::utils::math;
use crate::utils::reading::ReadFile;
use std::collections::HashMap;

pub(super) fn main_buffer_fields(
    files: &[ReadFile],
    vars: &[&VarDefinition],
    state: &TranspileState<'_, '_>,
) -> (HashMap<String, BufferField>, u32) {
    let mut offset = 0;
    let fields = vars
        .iter()
        .enumerate()
        .map(|(index, var)| {
            let dot_path = &files[var.name_span.file_index].dot_path;
            let path = format!("{}:{}", dot_path, var.name);
            let type_ = types::var_type(var, state.inner)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"));
            let field = BufferField {
                type_id: type_.id,
                size: type_.size(),
                offset,
            };
            offset = main_buffer_next_field_offset(vars, index, offset, type_, state);
            (path, field)
        })
        .collect::<HashMap<_, _>>();
    (fields, offset)
}

pub(super) fn main_buffer_alignment(
    vars: &[&VarDefinition],
    state: &TranspileState<'_, '_>,
) -> u32 {
    vars.iter()
        .map(|var| {
            types::var_type(var, state.inner)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"))
                .alignment()
        })
        .max()
        .unwrap_or(0)
}

fn main_buffer_next_field_offset(
    fields: &[&VarDefinition],
    current_field_index: usize,
    current_field_offset: u32,
    current_field_type: &StructDefinition,
    state: &TranspileState<'_, '_>,
) -> u32 {
    if let Some(next_var) = fields.get(current_field_index + 1) {
        let next_var_type = types::var_type(next_var, state.inner)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("variable type should be validated before"));
        math::round_up(
            next_var_type.alignment(),
            current_field_offset + current_field_type.size(),
        )
    } else {
        current_field_offset + current_field_type.size()
    }
}
