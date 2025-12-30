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
const I32_SIZE_BYTES: u32 = 4;

/// A compiled `GPEx` program.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Program {
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
    /// The size of the field in bytes.
    pub size: u32,
    /// The offset in bytes of the field inside its buffer.
    pub offset: u32,
}

pub(crate) fn transpile(files: &[ReadFile], modules: &[Module], indexes: &Indexes<'_>) -> Program {
    let mut init_shader = String::with_capacity(100);
    transpile_init(&mut init_shader, modules, indexes);
    let mut offset = 0;
    let fields = modules
        .iter()
        .flat_map(Module::global_variables)
        .sorted_unstable_by_key(|variable| variable.id)
        .map(|variable| {
            let dot_path = &files[variable.name_span.file_index].dot_path;
            let path = format!("{}:{}", dot_path, variable.name);
            let field = BufferField {
                size: I32_SIZE_BYTES,
                offset,
            };
            offset += field.size;
            (path, field)
        })
        .collect::<HashMap<_, _>>();
    Program {
        buffer: Buffer {
            size: offset,
            fields,
        },
        init_shader,
    }
}

fn transpile_init(shader: &mut String, modules: &[Module], indexes: &Indexes<'_>) {
    *shader += "struct Buffer { ";
    for module in modules {
        for variable in module.global_variables() {
            variable.transpile_buffer_field(shader);
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
