#![expect(clippy::multiple_inherent_impl)]

mod compilerimpl;
mod exprs;
mod items;

use self::compilerimpl::CompilerImplType;
use crate::compiler::dependencies::DependencyResolver;
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::{FnDefinition, FnStatementsBody};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::modules::Module;
use crate::compiler::values::ValueResolver;
use crate::compiler::values::consts::ConstValue;
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

#[derive(Debug)]
pub(crate) struct Transpiler<'item, 'index> {
    indexes: &'index Indexes<'item>,
    shader: String,
    value_resolver: ValueResolver<'item, 'index>,
    compilerimpl_types: HashMap<u64, CompilerImplType>,
    specialized_fns: HashMap<SpecializedFn<'item>, usize>,
    transpiled_specialized_fn_indexes: HashSet<usize>,
}

impl<'item, 'index> Transpiler<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            indexes,
            shader: String::new(),
            value_resolver: ValueResolver::new(indexes),
            compilerimpl_types: CompilerImplType::all_by_id(indexes),
            specialized_fns: HashMap::default(),
            transpiled_specialized_fn_indexes: HashSet::default(),
        }
    }

    pub(crate) fn transpile(&mut self, files: &[ReadFile], modules: &[Module]) -> Program {
        let init_shader = self.transpile_init(modules);
        let update_shader = self.transpile_repeats(modules);
        let mut offset = 0;
        let variables: Vec<_> = Self::sorted_global_vars_for_definition(modules);
        let buffer_alignment = self.main_buffer_alignment(&variables);
        let fields = variables
            .iter()
            .enumerate()
            .map(|(index, var)| {
                let dot_path = &files[var.name_span.file_index].dot_path;
                let path = format!("{}:{}", dot_path, var.name);
                let type_ = self
                    .value_resolver
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
        &mut self,
        fields: &[&VarDefinition],
        current_field_index: usize,
        current_field_offset: u32,
        current_field_type: &StructDefinition,
    ) -> u32 {
        if let Some(next_var) = fields.get(current_field_index + 1) {
            let next_var_type = self
                .value_resolver
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

    fn main_buffer_alignment(&mut self, vars: &[&VarDefinition]) -> u32 {
        vars.iter()
            .map(|var| {
                self.value_resolver
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

    fn transpile_init(&mut self, modules: &[Module]) -> String {
        self.transpile_shader(modules, |self_| {
            for var in self_.sorted_global_vars_for_init(modules) {
                self_.transpile_var_init(var);
            }
        });
        mem::take(&mut self.shader)
    }

    fn transpile_repeats(&mut self, modules: &[Module]) -> String {
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
        self.shader += " @compute @workgroup_size(1, 1, 1) fn main() { ";
        transpile_body(self);
        self.shader += "}";
        let mut last_fn_count = 0;
        while last_fn_count != self.specialized_fns.len() {
            last_fn_count = self.specialized_fns.len();
            for (fn_, index) in self
                .specialized_fns
                .clone()
                .into_iter()
                .sorted_by_key(|(_, index)| *index)
            {
                self.transpile_specialized_fn(fn_, index);
            }
        }
        self.specialized_fns.clear();
        self.transpiled_specialized_fn_indexes.clear();
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
        for var in Self::sorted_global_vars_for_definition(modules) {
            self.transpile_var_as_struct_field(var);
        }
        self.shader += "} @group(0) @binding(0) var<storage, read_write> ";
        self.shader += MAIN_BUFFER_NAME;
        self.shader += ": Buffer; ";
    }

    fn sorted_global_vars_for_init(&self, modules: &'item [Module]) -> Vec<&'item VarDefinition> {
        let mut dependency_graph = DiGraphMap::<&VarDefinition, ()>::new();
        for var in modules.iter().flat_map(Module::global_vars) {
            dependency_graph.add_node(var);
            let mut dependency_resolver = DependencyResolver::new(self.indexes);
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

    fn sorted_global_vars_for_definition(modules: &[Module]) -> Vec<&VarDefinition> {
        modules
            .iter()
            .flat_map(Module::global_vars)
            .sorted_unstable_by_key(|var| var.id)
            .collect()
    }
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
