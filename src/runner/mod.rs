mod resources;
mod utils;

use crate::compiler::transpilation::Program;
use crate::runner::resources::ComputeShader;
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
                msg: format!("invalid compilated program \"{}\"", path.display()),
                loc: None,
                inner: vec![],
            }]
        })?),
        Err(err) => Err(vec![Log::from_io_error(err, path, "cannot read")]),
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
        Ok(Self {
            program,
            device,
            queue,
            buffer,
            init_shader,
        })
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
            Some(GpuValue::I32(i32::from_ne_bytes([
                buffer[0], buffer[1], buffer[2], buffer[3],
            ])))
        } else {
            None
        }
    }

    /// Runs a program step.
    pub fn run_step(&mut self) {
        let mut encoder = utils::create_encoder(&self.device);
        if let Some(init_shader) = &mut self.init_shader
            && !init_shader.is_init_done
        {
            let mut pass = utils::start_compute_pass(&mut encoder);
            init_shader.run(&mut pass);
        }
        self.queue.submit(Some(encoder.finish()));
    }
}

/// A value retrieved from GPU.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GpuValue {
    /// An `i32` value.
    I32(i32),
}

impl Display for GpuValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I32(value) => Display::fmt(value, f),
        }
    }
}
