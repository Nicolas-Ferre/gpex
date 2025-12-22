use crate::compiler::indexes::Indexes;
use crate::compiletools::indexing::Node;
use crate::compiletools::reading::ReadFile;
use crate::language::module::Module;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) const MAIN_BUFFER_NAME: &str = "b";
const VAR_BYTES: u32 = 4;

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
        .flat_map(|module| &module.items)
        .sorted_unstable_by_key(|var| var.id())
        .map(|var| {
            let dot_path = &files[var.file_index()].dot_path;
            let path = format!("{}:{}", dot_path, var.name());
            let field = BufferField {
                size: VAR_BYTES,
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
        module.transpile_buffer_fields(shader);
    }
    *shader += "} @group(0) @binding(0) var<storage, read_write> ";
    *shader += MAIN_BUFFER_NAME;
    *shader += ": Buffer; ";
    *shader += "@compute @workgroup_size(1, 1, 1) fn main() { ";
    for module in modules {
        module.transpile_buffer_init(shader, indexes);
    }
    *shader += "}";
}
