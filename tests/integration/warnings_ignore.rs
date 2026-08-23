use crate::files;
use gpex::{Log, LogLevel};
use libtest_mimic::Failed;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn is_ignored_warning(log: &Log, root_path: &Path, ignored_paths: &[PathBuf]) -> bool {
    if log.level != LogLevel::Warning {
        return false;
    }
    let Some(location) = log.location.as_ref() else {
        return false;
    };
    let relative_path = location
        .path
        .strip_prefix(root_path)
        .unwrap_or(location.path.as_path());
    ignored_paths
        .iter()
        .any(|ignored_path| ignored_path == relative_path)
}

pub(crate) fn warning_ignore_paths(path: &Path) -> Result<Vec<PathBuf>, Failed> {
    let ignore_path = path.join(".warningsignore");
    if !ignore_path.exists() {
        return Ok(vec![]);
    }
    fs::read_to_string(&ignore_path)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .map(|relative_path| validate_warning_ignored_path(path, relative_path))
        .collect()
}

fn validate_warning_ignored_path(path: &Path, relative_path: PathBuf) -> Result<PathBuf, Failed> {
    if files::is_file_gpex(&relative_path) && path.join(&relative_path).is_file() {
        Ok(relative_path)
    } else {
        Err(format!(
            "entry in .warningsignore does not identify an existing .gpex file: {}",
            relative_path.display()
        )
        .into())
    }
}
