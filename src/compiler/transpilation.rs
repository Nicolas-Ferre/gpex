use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::language::items::var::VariableDefinition;
use crate::language::module::Module;
use crate::utils::reading::ReadFile;
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) const MAIN_BUFFER_NAME: &str = "b";

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

pub(crate) fn transpile(files: &[ReadFile], modules: &[Module], indexes: &Indexes<'_>) -> Program {
    let mut init_shader = String::with_capacity(100);
    transpile_init(&mut init_shader, modules, indexes);
    let mut offset = 0;
    let variables: Vec<_> = modules.iter().flat_map(Module::global_variables).collect();
    let buffer_alignment = variables
        .iter()
        .map(|variable| variable.type_(indexes).alignment())
        .max()
        .unwrap_or(0);
    let fields = variables
        .iter()
        .enumerate()
        .sorted_unstable_by_key(|(_, variable)| variable.id)
        .map(|(index, variable)| {
            let dot_path = &files[variable.name_span.file_index].dot_path;
            let path = format!("{}:{}", dot_path, variable.name);
            let type_ = variable.type_(indexes);
            let field = BufferField {
                type_id: type_.id,
                size: type_.size(),
                offset,
            };
            offset = if let Some(next_variable) = variables.get(index + 1) {
                round_up(
                    next_variable.type_(indexes).alignment(),
                    offset + type_.size(),
                )
            } else {
                offset + type_.size()
            };
            (path, field)
        })
        .collect::<HashMap<_, _>>();
    Program {
        type_paths: indexes
            .types
            .iter()
            .copied()
            .chain(variables.iter().map(|variables| variables.type_(indexes)))
            .map(|type_| (type_.id, type_.dot_path()))
            .collect(),
        buffer: Buffer {
            size: round_up(buffer_alignment, offset),
            fields,
        },
        init_shader,
    }
}

fn transpile_init(shader: &mut String, modules: &[Module], indexes: &Indexes<'_>) {
    *shader += "struct Buffer { ";
    for module in modules {
        for variable in module.global_variables() {
            variable.transpile_buffer_field(shader, indexes);
        }
    }
    *shader += "} @group(0) @binding(0) var<storage, read_write> ";
    *shader += MAIN_BUFFER_NAME;
    *shader += ": Buffer; ";
    *shader += "@compute @workgroup_size(1, 1, 1) fn main() { ";
    for variable in sorted_global_variables(modules, indexes) {
        variable.transpile_buffer_init(shader, indexes);
    }
    *shader += "}";
}

#[expect(clippy::expect_used)] // circular dependencies checked during validation phase
fn sorted_global_variables<'items>(
    modules: &'items [Module],
    indexes: &Indexes<'items>,
) -> Vec<&'items VariableDefinition> {
    let mut dependency_graph = DiGraphMap::<&VariableDefinition, ()>::new();
    for variable in modules.iter().flat_map(Module::global_variables) {
        dependency_graph.add_node(variable);
        let dependencies = variable
            .dependencies(Dependencies::new(ItemRef::Variable(variable)), indexes)
            .expect("internal error: found circular dependencies");
        for dependency in dependencies.into_iter() {
            if let ItemRef::Variable(dependency) = dependency {
                dependency_graph.add_edge(dependency, variable, ());
            }
        }
    }
    petgraph::algo::toposort(&dependency_graph, None)
        .expect("internal error: found circular dependencies")
}

fn round_up(rounded_to: u32, value: u32) -> u32 {
    if rounded_to == 0 {
        0
    } else {
        value.div_ceil(rounded_to) * rounded_to
    }
}
