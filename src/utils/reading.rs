use crate::compiler::prelude;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::utils::logs::Log;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug)]
pub(crate) struct ReadFile {
    pub(crate) content: String,
    pub(crate) fs_path: PathBuf,
    pub(crate) dot_path: String,
}

#[expect(clippy::absurd_extreme_comparisons)]
pub(crate) fn read(path: &Path, extension: &str) -> Result<Vec<ReadFile>, Vec<Log>> {
    let mut files = read_folder(path, path, extension)?;
    debug_assert!(files.len() >= PRELUDE_FILE_INDEX);
    files.insert(PRELUDE_FILE_INDEX, prelude::file());
    Ok(files)
}

fn read_folder(path: &Path, root_path: &Path, extension: &str) -> Result<Vec<ReadFile>, Vec<Log>> {
    let mut files = vec![];
    let mut errors = vec![];
    for entry in fs::read_dir(path).map_err(|error| to_log(error, path))? {
        let entry = entry.map_err(|error| to_log(error, path))?;
        match read_entry(entry, root_path, extension) {
            Ok(new_files) => files.extend(new_files),
            Err(new_errors) => errors.extend(new_errors), // no-coverage (difficult to test)
        }
    }
    if errors.is_empty() {
        files.sort_unstable_by(|file1, file2| file1.fs_path.cmp(&file2.fs_path));
        Ok(files)
    } else {
        Err(errors) // no-coverage (difficult to test)
    }
}

fn read_entry(
    entry: DirEntry,
    root_path: &Path,
    extension: &str,
) -> Result<Vec<ReadFile>, Vec<Log>> {
    let path = entry.path();
    let file_type = entry.file_type().map_err(|error| to_log(error, &path))?;
    if file_type.is_dir() {
        read_folder(&path, root_path, extension)
    } else if path.extension() == Some(OsStr::new(extension)) {
        let content = fs::read_to_string(&path).map_err(|error| to_log(error, &path))?;
        Ok(vec![ReadFile {
            content,
            dot_path: dot_path(&path, root_path),
            fs_path: path,
        }])
    } else {
        Ok(vec![])
    }
}

fn dot_path(path: &Path, root_path: &Path) -> String {
    path.with_extension("")
        .strip_prefix(root_path)
        .unwrap_or_else(|_| unreachable!("path should always be obtained from root path"))
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .join(".")
}

fn to_log(error: io::Error, path: &Path) -> Vec<Log> {
    vec![Log::from_io_error(error, path, "cannot read")]
}
