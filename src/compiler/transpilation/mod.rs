mod exprs;
mod items;

use crate::compiler::consts::ConstResolver;
use crate::compiler::dependencies::{DependencyResolver, DependencyType};
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::modules::Module;
use crate::compiler::types::TypeResolver;
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

pub(crate) struct Transpiler<'item, 'index> {
    indexes: &'index Indexes<'item>,
    shader: String,
    dependencies: Dependencies<ItemRef<'item>>,
    type_resolver: TypeResolver<'item, 'index>,
    const_checker: ConstResolver<'item, 'index>,
}

impl<'item, 'index> Transpiler<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            indexes,
            shader: String::new(),
            dependencies: Dependencies::new(),
            type_resolver: TypeResolver::new(indexes),
            const_checker: ConstResolver::new(indexes),
        }
    }

    pub(crate) fn transpile(&mut self, files: &[ReadFile], modules: &[Module]) -> Program {
        let init_shader = self.transpile_init(modules);
        let update_shader = self.transpile_repeats(modules);
        let mut offset = 0;
        let variables: Vec<_> = modules
            .iter()
            .flat_map(Module::global_vars)
            .sorted_unstable_by_key(|var| var.id)
            .collect();
        let buffer_alignment = self.main_buffer_alignment(&variables);
        let fields = variables
            .iter()
            .enumerate()
            .map(|(index, var)| {
                let dot_path = &files[var.name_span.file_index].dot_path;
                let path = format!("{}:{}", dot_path, var.name);
                let type_ = self
                    .type_resolver
                    .var_type(var)
                    .struct_ref()
                    .unwrap_or_else(|| unreachable!("variable type should be validated before"));
                let field = BufferField {
                    type_id: type_.id,
                    size: type_.size(),
                    offset,
                };
                offset = self.main_buffer_next_field_offset(&variables, index, offset, type_);
                (path, field)
            })
            .collect::<HashMap<_, _>>();
        Program {
            type_paths: self.type_paths(),
            buffer: Buffer {
                size: Self::round_up(buffer_alignment, offset),
                fields,
            },
            init_shader,
            update_shader,
        }
    }

    fn main_buffer_next_field_offset(
        &self,
        fields: &[&VarDefinition],
        current_field_index: usize,
        current_field_offset: u32,
        current_field_type: &StructDefinition,
    ) -> u32 {
        if let Some(next_var) = fields.get(current_field_index + 1) {
            let next_var_type = self
                .type_resolver
                .var_type(next_var)
                .struct_ref()
                .unwrap_or_else(|| unreachable!("variable type should be validated before"));
            Self::round_up(
                next_var_type.alignment(),
                current_field_offset + current_field_type.size(),
            )
        } else {
            current_field_offset + current_field_type.size()
        }
    }

    fn main_buffer_alignment(&self, vars: &[&VarDefinition]) -> u32 {
        vars.iter()
            .map(|var| {
                self.type_resolver
                    .var_type(var)
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

    fn type_paths(&self) -> HashMap<u64, String> {
        self.indexes
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

    pub(crate) fn transpile_init(&mut self, modules: &[Module]) -> String {
        let mut dependency_resolver =
            DependencyResolver::new(DependencyType::Transpilation, self.indexes);
        for module in modules {
            for var in module.global_vars() {
                dependency_resolver.scan_var(var).unwrap_or_else(|_| {
                    unreachable!("circular dependencies should be validated before")
                });
            }
        }
        self.dependencies = dependency_resolver.dependencies;
        self.transpile_shader(modules, |self_| {
            for var in self_.sorted_global_vars(modules) {
                self_.transpile_var_init(var);
            }
        });
        mem::take(&mut self.shader)
    }

    pub(crate) fn transpile_repeats(&mut self, modules: &[Module]) -> String {
        let mut dependency_resolver =
            DependencyResolver::new(DependencyType::Transpilation, self.indexes);
        for module in modules {
            for repeat in module.repeats() {
                dependency_resolver.scan_repeat(repeat).unwrap_or_else(|_| {
                    unreachable!("circular dependencies should be validated before")
                });
            }
        }
        self.dependencies = dependency_resolver.dependencies;
        self.transpile_shader(modules, |self_| {
            for module in modules {
                for repeat in module.repeats() {
                    self_.transpile_repeat(repeat);
                }
            }
        });
        mem::take(&mut self.shader)
    }

    fn transpile_shader(&mut self, modules: &[Module], transpile_body: impl FnOnce(&mut Self)) {
        self.transpile_buffer_header(modules);
        for dependency in self.dependencies.clone().iter() {
            self.transpile_item(dependency);
        }
        self.shader += " @compute @workgroup_size(1, 1, 1) fn main() { ";
        transpile_body(self);
        self.shader += "}";
    }

    fn transpile_buffer_header(&mut self, modules: &[Module]) {
        let is_buffer_empty = modules
            .iter()
            .flat_map(Module::global_vars)
            .next()
            .is_none();
        if is_buffer_empty {
            return;
        }
        self.shader += "struct Buffer { ";
        for module in modules {
            for var in module.global_vars() {
                self.transpile_var_as_struct_field(var);
            }
        }
        self.shader += "} @group(0) @binding(0) var<storage, read_write> ";
        self.shader += MAIN_BUFFER_NAME;
        self.shader += ": Buffer; ";
    }

    fn sorted_global_vars(&self, modules: &'item [Module]) -> Vec<&'item VarDefinition> {
        let mut dependency_graph = DiGraphMap::<&VarDefinition, ()>::new();
        for var in modules.iter().flat_map(Module::global_vars) {
            dependency_graph.add_node(var);
            let mut dependency_resolver =
                DependencyResolver::new(DependencyType::Transpilation, self.indexes);
            dependency_resolver.scan_var(var).unwrap_or_else(|_| {
                unreachable!("circular dependencies should be validated before")
            });
            for dependency in dependency_resolver.dependencies.iter() {
                if let ItemRef::Var(dependency) = dependency {
                    dependency_graph.add_edge(dependency, var, ());
                }
            }
        }
        petgraph::algo::toposort(&dependency_graph, None)
            .unwrap_or_else(|_| unreachable!("circular dependencies should be validated before"))
    }
}
