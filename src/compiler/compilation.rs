use crate::compiler::indexes::Indexes;
use crate::compiletools::logs::{Log, LogLevel};
use crate::compiletools::parsing::ParseCtx;
use crate::compiletools::reading::ReadFile;
use crate::compiletools::validation::ValidateCtx;
use crate::language::module::Module;
use std::path::Path;

const COMMENT_PREFIX: &str = "//";

pub(crate) fn parse(files: &[ReadFile]) -> Result<Vec<Module>, Vec<Log>> {
    let mut next_id = 0;
    let mut modules = vec![];
    let mut errors = vec![];
    for (file_index, file) in files.iter().enumerate() {
        let mut ctx = ParseCtx::new(file, file_index, files, next_id, COMMENT_PREFIX);
        match Module::parse(&mut ctx) {
            Ok(module) => modules.push(module),
            Err(err) => errors.push(err.to_error()),
        }
        next_id = ctx.next_id();
    }
    if errors.is_empty() {
        Ok(modules)
    } else {
        Err(errors)
    }
}

pub(crate) fn index(modules: &[Module]) -> Indexes<'_> {
    let mut indexes = Indexes::new(modules.len());
    for module in modules {
        module.index(&mut indexes);
    }
    indexes.imports.consolidate();
    indexes
}

pub(crate) fn validate(
    root_path: &Path,
    files: &[ReadFile],
    modules: &[Module],
    indexes: &mut Indexes<'_>,
    warnings_as_errors: bool,
) -> Result<Vec<Log>, Vec<Log>> {
    let mut ctx = ValidateCtx::new(files, root_path);
    for module in modules {
        module.pre_validate(indexes);
    }
    for module in modules {
        module.validate(&mut ctx, indexes);
    }
    if ctx.logs.iter().any(|log| {
        log.level == LogLevel::Error || (warnings_as_errors && log.level == LogLevel::Warning)
    }) {
        Err(ctx.logs)
    } else {
        Ok(ctx.logs)
    }
}
