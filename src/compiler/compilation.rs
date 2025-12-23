use crate::compiler::indexes::Indexes;
use crate::compiletools::logs::{Log, LogLevel};
use crate::compiletools::parsing::ParseCtx;
use crate::compiletools::reading::ReadFile;
use crate::compiletools::validation::ValidateCtx;
use crate::language::module::Module;

const COMMENT_PREFIX: &str = "//";

pub(crate) fn parse(files: &[ReadFile]) -> Result<Vec<Module>, Vec<Log>> {
    let mut next_id = 0;
    let mut modules = vec![];
    let mut errors = vec![];
    for (file_index, file) in files.iter().enumerate() {
        let mut ctx = ParseCtx::new(file, file_index, next_id, COMMENT_PREFIX);
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
    indexes
}

pub(crate) fn validate(
    files: &[ReadFile],
    modules: &[Module],
    indexes: &mut Indexes<'_>,
    warnings_as_error: bool,
) -> Result<Vec<Log>, Vec<Log>> {
    let mut errors = vec![];
    for module in modules {
        module.pre_validate(indexes);
    }

    for module in modules {
        let mut ctx = ValidateCtx::new(files);
        module.validate(&mut ctx, indexes);
        errors.extend(ctx.logs);
    }
    if errors.iter().any(|log| {
        log.level == LogLevel::Error || (warnings_as_error && log.level == LogLevel::Warning)
    }) {
        Err(errors)
    } else {
        Ok(errors)
    }
}
