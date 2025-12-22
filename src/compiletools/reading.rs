use crate::compiletools::logs::Log;
use itertools::Itertools;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug)]
pub(crate) struct ReadFile {
    pub(crate) content: String,
    pub(crate) path: PathBuf,
    pub(crate) dot_path: String,
}

pub(crate) fn read(path: &Path, root_path: &Path, ext: &str) -> Result<Vec<ReadFile>, Vec<Log>> {
    let mut files = vec![];
    let mut errors = vec![];
    for entry in fs::read_dir(path).map_err(|err| to_log(err, path))? {
        let entry = entry.map_err(|err| to_log(err, path))?;
        match read_entry(entry, root_path, ext) {
            Ok(new_files) => files.extend(new_files),
            Err(new_errors) => errors.extend(new_errors), // no-coverage (difficult to test)
        }
    }
    if errors.is_empty() {
        files.sort_unstable_by(|file1, file2| file1.path.cmp(&file2.path));
        Ok(files)
    } else {
        Err(errors) // no-coverage (difficult to test)
    }
}

fn read_entry(entry: DirEntry, root_path: &Path, ext: &str) -> Result<Vec<ReadFile>, Vec<Log>> {
    let path = entry.path();
    let file_type = entry.file_type().map_err(|err| to_log(err, &path))?;
    if file_type.is_dir() {
        read(&path, root_path, ext)
    } else if path.extension() == Some(OsStr::new(ext)) {
        let content = fs::read_to_string(&path).map_err(|err| to_log(err, &path))?;
        Ok(vec![ReadFile {
            content,
            dot_path: dot_path(&path, root_path),
            path,
        }])
    } else {
        Ok(vec![])
    }
}

fn dot_path(path: &Path, root_path: &Path) -> String {
    #[expect(clippy::unwrap_used)] // path is always obtained from root_path
    path.with_extension("")
        .strip_prefix(root_path)
        .unwrap()
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .join(".")
}

fn to_log(err: io::Error, path: &Path) -> Vec<Log> {
    vec![Log::from_io_error(err, path, "cannot read")]
}
