pub(crate) mod consts;
pub(crate) mod dependencies;
pub(crate) mod indexing;
pub(crate) mod key_rendering;
pub(crate) mod parsing;
pub(crate) mod prelude;
pub(crate) mod refs;
pub(crate) mod transpilation;
pub(crate) mod validation;
pub(crate) mod values;

use crate::compiler::indexing::indexer::Indexer;
use crate::compiler::transpilation::{Program, Transpiler};
use crate::compiler::validation::Validator;
use crate::utils::logs::Log;
use crate::utils::reading;
use std::fs;
use std::path::Path;

pub(crate) const EXT: &str = "gpex";

/// Compiles a `GPEx` project folder.
///
/// # Errors
///
/// An error is returned in case compilation fails.
pub fn compile_program(
    root_path: &Path,
    is_warning_treated_as_error: bool,
) -> Result<(Program, Vec<Log>), Vec<Log>> {
    let files = reading::read(root_path, EXT)?;
    let modules = parsing::parse(root_path, &files)?;
    let indexes = Indexer::run(&modules);
    let errors = Validator::new(&files, root_path, &indexes)
        .validate_modules(&modules, is_warning_treated_as_error)?;
    let program = Transpiler::new(&indexes).transpile(&files, &modules);
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
