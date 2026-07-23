pub(crate) mod compilerimpl;
mod exprs;
mod items;

use crate::compiler::dependencies;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::{FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::modules::Module;
use crate::compiler::state::State;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types;
use crate::utils::dependencies::Dependencies;
use crate::utils::reading::ReadFile;
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;

const MAIN_BUFFER_NAME: &str = "b";

/// A compiled `GPEx` program.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Program {
    /// For each type ID, the dot path of the type.
    pub type_paths: HashMap<u64, String>,
    /// The buffer storing all global variables.
    pub buffer: Buffer,
    /// The shader used to initialize all global variables.
    pub init_shader: String,
    /// The shader used to update application at each frame.
    pub update_shader: String,
}

/// A buffer in a `GPEx` program.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Buffer {
    /// The size of the buffer in bytes.
    pub size: u32,
    /// The fields of the buffer.
    pub fields: HashMap<String, BufferField>,
}

/// A buffer field in a `GPEx` program.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BufferField {
    /// The field type ID.
    pub type_id: u64,
    /// The size of the field in bytes.
    pub size: u32,
    /// The offset in bytes of the field inside its buffer.
    pub offset: u32,
}

pub(crate) fn transpile<'item>(
    files: &[ReadFile],
    modules: &'item [Module],
    state: &mut State<'item>,
) -> Program {
    let init_shader = transpile_init(modules, state);
    let update_shader = transpile_repeats(modules, state);
    let mut offset = 0;
    let variables: Vec<_> = sorted_global_vars_for_definition(modules);
    let buffer_alignment = main_buffer_alignment(&variables, state);
    let fields = variables
        .iter()
        .enumerate()
        .map(|(index, var)| {
            let dot_path = &files[var.name_span.file_index].dot_path;
            let path = format!("{}:{}", dot_path, var.name);
            let type_ = types::var_type(var, state)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"));
            let field = BufferField {
                type_id: type_.id,
                size: type_.size(),
                offset,
            };
            offset = main_buffer_next_field_offset(&variables, index, offset, type_, state);
            (path, field)
        })
        .collect::<HashMap<_, _>>();
    Program {
        type_paths: type_paths(state),
        buffer: Buffer {
            size: round_up(buffer_alignment, offset),
            fields,
        },
        init_shader,
        update_shader,
    }
}

fn main_buffer_next_field_offset(
    fields: &[&VarDefinition],
    current_field_index: usize,
    current_field_offset: u32,
    current_field_type: &StructDefinition,
    state: &State<'_>,
) -> u32 {
    if let Some(next_var) = fields.get(current_field_index + 1) {
        let next_var_type = types::var_type(next_var, state)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("variable type should be validated before"));
        round_up(
            next_var_type.alignment(),
            current_field_offset + current_field_type.size(),
        )
    } else {
        current_field_offset + current_field_type.size()
    }
}

fn main_buffer_alignment(vars: &[&VarDefinition], state: &State<'_>) -> u32 {
    vars.iter()
        .map(|var| {
            types::var_type(var, state)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"))
                .alignment()
        })
        .max()
        .unwrap_or(0)
}

fn round_up(rounded_to: u32, value: u32) -> u32 {
    if rounded_to == 0 {
        0
    } else {
        value.div_ceil(rounded_to) * rounded_to
    }
}

fn type_paths(state: &State<'_>) -> HashMap<u64, String> {
    state
        .items
        .iter()
        .filter_map(|item| {
            if let ItemRef::Struct(type_) = item {
                Some((type_.id, type_.dot_path()))
            } else {
                None
            }
        })
        .collect()
}

fn transpile_init<'item>(modules: &'item [Module], state: &mut State<'item>) -> String {
    transpile_shader(
        modules,
        |state_| {
            for var in sorted_global_vars_for_init(modules, state_) {
                items::transpile_var_init(var, state_);
            }
        },
        state,
    );
    mem::take(&mut state.shader)
}

fn transpile_repeats<'item>(modules: &'item [Module], state: &mut State<'item>) -> String {
    transpile_shader(
        modules,
        |state_| {
            for module in modules {
                for repeat in module.repeats() {
                    items::transpile_repeat(repeat, state_);
                }
            }
        },
        state,
    );
    mem::take(&mut state.shader)
}

fn transpile_shader<'item>(
    modules: &[Module],
    transpile_body: impl FnOnce(&mut State<'item>),
    state: &mut State<'item>,
) {
    transpile_buffer_header(modules, state);
    state.shader += " @compute @workgroup_size(1, 1, 1) fn main() { ";
    transpile_body(state);
    state.shader += "}";
    let mut last_fn_count = 0;
    while last_fn_count != state.specialized_fns.len() {
        last_fn_count = state.specialized_fns.len();
        for (fn_, index) in state
            .specialized_fns
            .clone()
            .into_iter()
            .sorted_by_key(|(_, index)| *index)
        {
            items::transpile_specialized_fn(fn_, index, state);
        }
    }
    state.specialized_fns.clear();
    state.transpiled_specialized_fn_indexes.clear();
}

fn transpile_buffer_header(modules: &[Module], state: &mut State<'_>) {
    let is_buffer_empty = modules
        .iter()
        .flat_map(Module::global_vars)
        .next()
        .is_none();
    if is_buffer_empty {
        return;
    }
    state.shader += "struct Buffer { ";
    for var in sorted_global_vars_for_definition(modules) {
        items::transpile_var_as_struct_field(var, state);
    }
    state.shader += "} @group(0) @binding(0) var<storage, read_write> ";
    state.shader += MAIN_BUFFER_NAME;
    state.shader += ": Buffer; ";
}

fn sorted_global_vars_for_init<'item>(
    modules: &'item [Module],
    state: &mut State<'item>,
) -> Vec<&'item VarDefinition> {
    let mut dependency_graph = DiGraphMap::<&VarDefinition, ()>::new();
    for var in modules.iter().flat_map(Module::global_vars) {
        dependency_graph.add_node(var);
        let mut dependencies = Dependencies::new();
        dependencies::scan_var(var, &mut dependencies, state)
            .unwrap_or_else(|_| unreachable!("circular dependencies should be validated before"));
        for dependency in dependencies.iter() {
            if let ItemRef::Var(dependency) = dependency {
                dependency_graph.add_edge(dependency, var, ());
            }
        }
    }
    petgraph::algo::toposort(&dependency_graph, None)
        .unwrap_or_else(|_| unreachable!("circular dependencies should be validated before"))
}

fn sorted_global_vars_for_definition(modules: &[Module]) -> Vec<&VarDefinition> {
    modules
        .iter()
        .flat_map(Module::global_vars)
        .sorted_unstable_by_key(|var| var.id)
        .collect()
}

#[derive(Debug, Clone)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct SpecializedFn<'item> {
    fn_: &'item FnDefinition,
    const_param_values: Vec<ConstValue<'item>>,
    wildcard_param_types: Vec<&'item StructDefinition>,
    #[derive_where(skip)]
    fn_body: &'item FnStatementsBody,
}
