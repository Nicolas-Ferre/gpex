mod resources;
mod utils;

use crate::compiler::parsing::symbols::{FALSE_KEYWORD, TRUE_KEYWORD};
use crate::compiler::transpilation::Program;
use crate::runner::resources::ComputeShader;
use crate::utils::{endianness, formatting};
use crate::{Log, LogLevel};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use wgpu::{Buffer, Device, Queue};

/// Loads a compiled `GPEx` program.
///
/// # Errors
///
/// An error is returned in case the input file is not a valid compiled `GPEx` program.
pub fn load_compiled(path: &Path) -> Result<Program, Vec<Log>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(serde_json::from_str(&content).map_err(|_| {
            vec![Log {
                level: LogLevel::Error,
                msg: format!("invalid compiled program \"{}\"", path.display()),
                location: None,
                inner: vec![],
            }]
        })?),
        Err(error) => Err(vec![Log::from_io_error(error, path, "cannot read")]),
    }
}

/// A `GPEx` program runner.
#[derive(Debug)]
pub struct Runner {
    program: Program,
    device: Device,
    queue: Queue,
    buffer: Option<Buffer>,
    init_shader: Option<ComputeShader>,
    update_shader: Option<ComputeShader>,
}

impl Runner {
    /// Creates a new runner.
    ///
    /// # Errors
    ///
    /// An error is returned in case the program cannot be initialized.
    pub async fn new(program: Program) -> Result<Self, Vec<Log>> {
        let instance = utils::create_instance();
        let adapter = utils::create_adapter(&instance).await?;
        let (device, queue) = utils::create_device(&adapter).await?;
        let buffer = utils::create_buffer(&device, "gpex:buffer:main", program.buffer.size.into());
        let init_shader = buffer
            .as_ref()
            .map(|buffer| ComputeShader::new(&device, buffer, &program.init_shader));
        let update_shader = buffer
            .as_ref()
            .map(|buffer| ComputeShader::new(&device, buffer, &program.update_shader));
        Ok(Self {
            program,
            device,
            queue,
            buffer,
            init_shader,
            update_shader,
        })
    }

    /// Returns information about the program.
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Reads global variable value.
    ///
    /// Variable `path` is the dot path of the module and the variable name separated by a `:`
    /// (e.g. `inner.module:my_buffer`).
    ///
    /// If the buffer doesn't exist, an empty vector is returned.
    pub fn read_var(&self, path: &str) -> Option<GpuValue> {
        if let Some(buffer) = self.buffer.as_ref()
            && let Some(field) = self.program.buffer.fields.get(path)
        {
            let buffer = utils::read_buffer(
                &self.device,
                &self.queue,
                buffer,
                field.offset.into(),
                field.size.into(),
            );
            Some(self.gpu_value(self.program.type_paths[&field.type_id].as_str(), &buffer))
        } else {
            None
        }
    }

    /// Runs a program step.
    pub fn run_step(&mut self) {
        let mut encoder = utils::create_encoder(&self.device);
        if let Some(shader) = &mut self.init_shader
            && !shader.is_init_done
        {
            let mut pass = utils::start_compute_pass(&mut encoder);
            shader.run(&mut pass);
        }
        if let Some(shader) = &mut self.update_shader {
            let mut pass = utils::start_compute_pass(&mut encoder);
            shader.run(&mut pass);
        }
        self.queue.submit(Some(encoder.finish()));
    }

    fn gpu_value(&self, type_path: &str, buffer: &[u8]) -> GpuValue {
        let bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
        match type_path {
            "i32" => GpuValue::I32(i32::from_ne_bytes(bytes)),
            "u32" => GpuValue::U32(u32::from_ne_bytes(bytes)),
            "f32" => GpuValue::F32(f32::from_ne_bytes(bytes)),
            "bool" => GpuValue::Bool(u32::from_ne_bytes(bytes) != 0),
            "typeref" => GpuValue::TypeRef(
                self.program.type_paths[&endianness::from_portable_u32x2(buffer)].clone(),
            ),
            _ => unreachable!("unrecognized GPU type"),
        }
    }
}

/// A value retrieved from GPU.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum GpuValue {
    /// A `typeref` value as dot path.
    TypeRef(String),
    /// An `i32` value.
    I32(i32),
    /// An `u32` value.
    U32(u32),
    /// An `f32` value.
    F32(f32),
    /// An `bool` value.
    Bool(bool),
}

impl Display for GpuValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeRef(path) => write!(formatter, "{path}"),
            Self::I32(value) => write!(formatter, "{value}"),
            Self::U32(value) => write!(formatter, "{value}u"),
            Self::F32(value) => write!(formatter, "{}", formatting::f32_to_string(*value)),
            Self::Bool(value) => write!(
                formatter,
                "{}",
                if *value {
                    TRUE_KEYWORD.slice
                } else {
                    FALSE_KEYWORD.slice
                }
            ),
        }
    }
}
