//! Compiled `GPEx` program.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
