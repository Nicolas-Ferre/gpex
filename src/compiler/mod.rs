pub(crate) mod compilation;
pub(crate) mod constants;
pub(crate) mod indexes;
pub(crate) mod prelude;
pub(crate) mod transpilation;
pub(crate) mod types;

use crate::compiler::transpilation::Program;
use crate::utils::logs::Log;
use crate::utils::reading;
use std::fs;
use std::path::Path;

pub(crate) const EXTENSION: &str = "gpex";

/// Compiles a `GPEx` project folder.
///
/// # Errors
///
/// An error is returned in case compilation fails.
pub fn compile(
    root_path: &Path,
    is_warning_treated_as_error: bool,
) -> Result<(Program, Vec<Log>), Vec<Log>> {
    let files = reading::read(root_path, EXTENSION)?;
    let modules = compilation::parse(root_path, &files)?;
    let indexes = compilation::index(&modules);
    let errors = compilation::validate(
        root_path,
        &files,
        &modules,
        &indexes,
        is_warning_treated_as_error,
    )?;
    let program = transpilation::transpile(&files, &modules, &indexes);
    Ok((program, errors))
}

/// Saves compiled `GPEx` program on disk.
///
/// # Errors
///
/// An error is returned in case the compiled program cannot be saved at the specified path.
pub fn save_compiled(program: &Program, path: &Path) -> Result<(), Vec<Log>> {
    let serialized = serde_json::to_string(&program)
        .unwrap_or_else(|_| unreachable!("JSON serialization of the program should never fail"));
    fs::write(path, serialized)
        .map_err(|error| vec![Log::from_io_error(error, path, "cannot write")])
}
