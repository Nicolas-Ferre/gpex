mod buffers;
mod exprs;
mod intrinsic;
mod items;

use crate::compiler::consts::ConstValue;
use crate::compiler::dependencies;
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::{FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::modules::Module;
use crate::compiler::state::State;
use crate::utils::dependencies::Dependencies;
use crate::utils::math;
use crate::utils::reading::ReadFile;
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
pub(crate) struct SpecializedFn<'item> {
    fn_: &'item FnDefinition,
    const_param_values: Vec<ConstValue<'item>>,
    wildcard_param_types: Vec<&'item StructDefinition>,
    #[derive_where(skip)]
    fn_body: &'item FnStatementsBody,
}

struct TranspileState<'state, 'item> {
    inner: &'state State<'item>,
    shader: String,
    specialized_fns: HashMap<SpecializedFn<'item>, usize>,
    transpiled_specialized_fn_indexes: HashSet<usize>,
}

impl<'state, 'item> TranspileState<'state, 'item> {
    fn new(state: &'state State<'item>) -> Self {
        Self {
            inner: state,
            shader: String::new(),
            specialized_fns: HashMap::new(),
            transpiled_specialized_fn_indexes: HashSet::new(),
        }
    }
}

pub(crate) fn transpile<'item>(
    files: &[ReadFile],
    modules: &'item [Module],
    state: &State<'item>,
) -> Program {
    let mut state = TranspileState::new(state);
    let init_shader = transpile_init(modules, &mut state);
    let update_shader = transpile_repeats(modules, &mut state);
    let vars: Vec<_> = sorted_global_vars_for_definition(modules);
    let buffer_alignment = buffers::main_buffer_alignment(&vars, &state);
    let (fields, buffer_size) = buffers::main_buffer_fields(files, &vars, &state);
    Program {
        type_paths: type_paths(&state),
        buffer: Buffer {
            size: math::round_up(buffer_alignment, buffer_size),
            fields,
        },
        init_shader,
        update_shader,
    }
}

fn type_paths(state: &TranspileState<'_, '_>) -> HashMap<u64, String> {
    state
        .inner
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

fn transpile_init<'item>(
    modules: &'item [Module],
    state: &mut TranspileState<'_, 'item>,
) -> String {
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

fn transpile_repeats<'item>(
    modules: &'item [Module],
    state: &mut TranspileState<'_, 'item>,
) -> String {
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

fn transpile_shader<'state, 'item>(
    modules: &[Module],
    transpile_body: impl FnOnce(&mut TranspileState<'state, 'item>),
    state: &mut TranspileState<'state, 'item>,
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

fn transpile_buffer_header(modules: &[Module], state: &mut TranspileState<'_, '_>) {
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
    state: &TranspileState<'_, 'item>,
) -> Vec<&'item VarDefinition> {
    let mut dependency_graph = DiGraphMap::<&VarDefinition, ()>::new();
    for var in modules.iter().flat_map(Module::global_vars) {
        dependency_graph.add_node(var);
        let mut dependencies = Dependencies::new();
        dependencies::scan_var(var, &mut dependencies, state.inner)
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
