use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::items::ItemRef;
use crate::language::items::struct_::StructDefinition;
use crate::language::items::variable::VariableDefinition;
use crate::language::module::Module;
use crate::utils::dependencies::Dependencies;
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

pub(crate) fn transpile(files: &[ReadFile], modules: &[Module], indexes: &Indexes<'_>) -> Program {
    let mut init_shader = String::with_capacity(100);
    transpile_init(&mut init_shader, modules, indexes);
    let mut update_shader = String::with_capacity(100);
    transpile_repeat(&mut update_shader, modules, indexes);
    let mut offset = 0;
    let variables: Vec<_> = modules
        .iter()
        .flat_map(Module::global_variables)
        .sorted_unstable_by_key(|variable| variable.id)
        .collect();
    let buffer_alignment = main_buffer_alignment(indexes, &variables);
    let fields = variables
        .iter()
        .enumerate()
        .map(|(index, variable)| {
            let dot_path = &files[variable.name_span.file_index].dot_path;
            let path = format!("{}:{}", dot_path, variable.name);
            let type_ = variable
                .type_(indexes)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"));
            let field = BufferField {
                type_id: type_.id,
                size: type_.size(),
                offset,
            };
            offset = main_buffer_next_field_offset(indexes, &variables, index, offset, type_);
            (path, field)
        })
        .collect::<HashMap<_, _>>();
    Program {
        type_paths: type_paths(indexes, &variables),
        buffer: Buffer {
            size: round_up(buffer_alignment, offset),
            fields,
        },
        init_shader,
        update_shader,
    }
}

fn main_buffer_next_field_offset(
    indexes: &Indexes<'_>,
    fields: &[&VariableDefinition],
    current_field_index: usize,
    current_field_offset: u32,
    current_field_type: &StructDefinition,
) -> u32 {
    if let Some(next_variable) = fields.get(current_field_index + 1) {
        let next_variable_type = next_variable
            .type_(indexes)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("variable type should be validated before"));
        round_up(
            next_variable_type.alignment(),
            current_field_offset + current_field_type.size(),
        )
    } else {
        current_field_offset + current_field_type.size()
    }
}

fn main_buffer_alignment(indexes: &Indexes<'_>, variables: &[&VariableDefinition]) -> u32 {
    variables
        .iter()
        .map(|variable| {
            variable
                .type_(indexes)
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

fn type_paths(indexes: &Indexes<'_>, variables: &[&VariableDefinition]) -> HashMap<u64, String> {
    indexes
        .types
        .iter()
        .copied()
        .chain(variables.iter().map(|variable| {
            variable
                .type_(indexes)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"))
        }))
        .map(|type_| (type_.id, type_.dot_path()))
        .collect()
}

fn transpile_init(shader: &mut String, modules: &[Module], indexes: &Indexes<'_>) {
    let mut dependencies = Dependencies::new();
    for module in modules {
        for variable in module.global_variables() {
            dependencies = variable
                .dependencies(DependencyType::Transpilation, dependencies, indexes)
                .unwrap_or_else(|_| {
                    unreachable!("circular dependencies should be validated before")
                });
        }
    }
    transpile_shader(shader, modules, indexes, dependencies, |shader| {
        for variable in sorted_global_variables(modules, indexes) {
            variable.transpile_buffer_init(shader, indexes);
        }
    });
}

fn transpile_repeat(shader: &mut String, modules: &[Module], indexes: &Indexes<'_>) {
    let mut dependencies = Dependencies::new();
    for module in modules {
        for repeat in module.repeats() {
            dependencies = repeat
                .function_call
                .dependencies(DependencyType::Transpilation, dependencies, indexes)
                .unwrap_or_else(|_| {
                    unreachable!("circular dependencies should be validated before")
                });
        }
    }
    transpile_shader(shader, modules, indexes, dependencies, |shader| {
        for module in modules {
            for repeat in module.repeats() {
                repeat.transpile_call(shader, indexes);
            }
        }
    });
}

fn transpile_shader<'item>(
    shader: &mut String,
    modules: &[Module],
    indexes: &Indexes<'item>,
    dependencies: Dependencies<ItemRef<'item>>,
    transpile_body: impl FnOnce(&mut String),
) {
    transpile_buffer_header(shader, modules, indexes);
    for dependency in dependencies.into_iter() {
        dependency.transpile(shader, indexes);
    }
    *shader += " @compute @workgroup_size(1, 1, 1) fn main() { ";
    transpile_body(shader);
    *shader += "}";
}

fn transpile_buffer_header(shader: &mut String, modules: &[Module], indexes: &Indexes<'_>) {
    *shader += "struct Buffer { ";
    for module in modules {
        for variable in module.global_variables() {
            variable.transpile_buffer_field(shader, indexes);
        }
    }
    *shader += "} @group(0) @binding(0) var<storage, read_write> ";
    *shader += MAIN_BUFFER_NAME;
    *shader += ": Buffer; ";
}

fn sorted_global_variables<'item>(
    modules: &'item [Module],
    indexes: &Indexes<'item>,
) -> Vec<&'item VariableDefinition> {
    let mut dependency_graph = DiGraphMap::<&VariableDefinition, ()>::new();
    for variable in modules.iter().flat_map(Module::global_variables) {
        dependency_graph.add_node(variable);
        let dependencies = variable
            .dependencies(DependencyType::Transpilation, Dependencies::new(), indexes)
            .unwrap_or_else(|_| unreachable!("circular dependencies should be validated before"));
        for dependency in dependencies.into_iter() {
            if let ItemRef::Variable(dependency) = dependency {
                dependency_graph.add_edge(dependency, variable, ());
            }
        }
    }
    petgraph::algo::toposort(&dependency_graph, None)
        .unwrap_or_else(|_| unreachable!("circular dependencies should be validated before"))
}
