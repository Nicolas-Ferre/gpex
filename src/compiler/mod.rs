pub(crate) mod compilation;
pub(crate) mod constants;
pub(crate) mod dependencies;
pub(crate) mod indexes;
pub(crate) mod transpilation;

use crate::compiler::transpilation::Program;
use crate::compiletools::logs::Log;
use crate::compiletools::reading;
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
    warnings_as_errors: bool,
) -> Result<(Program, Vec<Log>), Vec<Log>> {
    let files = reading::read(root_path, root_path, EXTENSION)?;
    let modules = compilation::parse(&files)?;
    let mut indexes = compilation::index(&modules);
    let errors = compilation::validate(
        root_path,
        &files,
        &modules,
        &mut indexes,
        warnings_as_errors,
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
    #[expect(clippy::unwrap_used)] // JSON serialization of the program never fails
    let serialized = serde_json::to_string(&program).unwrap();
    fs::write(path, serialized).map_err(|err| vec![Log::from_io_error(err, path, "cannot write")])
}
