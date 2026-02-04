use crate::compiler::indexes::Indexes;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::language::module::Module;
use crate::utils::logs::{Log, LogLevel};
use crate::utils::parsing::ParseContext;
use crate::utils::reading::ReadFile;
use crate::utils::validation::ValidateContext;
use petgraph::graphmap::DiGraphMap;
use std::path::Path;

const COMMENT_PREFIX: &str = "//";

pub(crate) fn parse(root_path: &Path, files: &[ReadFile]) -> Result<Vec<Module>, Vec<Log>> {
    let mut next_id = 0;
    let mut modules = vec![];
    let mut errors = vec![];
    for (file_index, file) in files.iter().enumerate() {
        let mut context =
            ParseContext::new(root_path, file, file_index, files, next_id, COMMENT_PREFIX);
        match Module::parse(&mut context) {
            Ok(module) => modules.push(module),
            Err(error) => errors.push(error.to_error()),
        }
        next_id = context.next_id();
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
        indexes
            .imports
            .register(None, None, module.file_index, PRELUDE_FILE_INDEX, false);
        module.index_imports(&mut indexes);
    }
    indexes.imports.consolidate();
    if let Some(sorted_modules) = sorted_modules_based_on_dependencies(modules, &indexes) {
        for module in &sorted_modules {
            module.index_items(&mut indexes);
        }
        for module in &sorted_modules {
            module.index_refs(&mut indexes);
        }
    } else {
        // do nothing, as the validation step will detect the cyclic imports
    }
    indexes
}

fn sorted_modules_based_on_dependencies<'item>(
    modules: &'item [Module],
    indexes: &Indexes<'item>,
) -> Option<Vec<&'item Module>> {
    let mut dependency_graph = DiGraphMap::<&Module, ()>::new();
    for module in modules {
        dependency_graph.add_node(module);
        for import in indexes.imports.imports(module.file_index) {
            if import.file_index == module.file_index {
                continue;
            }
            let imported_module = &modules[import.file_index];
            dependency_graph.add_edge(imported_module, module, ());
        }
    }
    petgraph::algo::toposort(&dependency_graph, None).ok()
}

pub(crate) fn validate(
    root_path: &Path,
    files: &[ReadFile],
    modules: &[Module],
    indexes: &Indexes<'_>,
    is_warning_treated_as_error: bool,
) -> Result<Vec<Log>, Vec<Log>> {
    let mut context = ValidateContext::new(files, root_path);
    for module in modules {
        _ = module.validate(&mut context, indexes);
    }
    if context
        .logs
        .iter()
        .any(|log| is_log_error(log, is_warning_treated_as_error))
    {
        Err(context.logs)
    } else {
        Ok(context.logs)
    }
}

fn is_log_error(log: &Log, is_warning_treated_as_error: bool) -> bool {
    log.level == LogLevel::Error || (is_warning_treated_as_error && log.level == LogLevel::Warning)
}
