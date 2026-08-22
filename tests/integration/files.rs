use itertools::Itertools;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

const GPEX_EXT: &str = "gpex";

pub(crate) fn path_parent(path: &Path) -> &Path {
    path.parent()
        .unwrap_or_else(|| unreachable!("parent should be at least temporary folder"))
}

pub(crate) fn path_file_name(path: &Path) -> Cow<'_, str> {
    path.file_name().unwrap_or_default().to_string_lossy()
}

pub(crate) fn is_dir_containing_gpex_file(path: &Path) -> Result<bool, IoError> {
    for entry in fs::read_dir(path)? {
        if is_file_gpex(&entry?.path()) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn is_file_gpex(path: &Path) -> bool {
    path.extension() == Some(OsStr::new(GPEX_EXT))
}

pub(crate) fn to_dot_path(file_path: &Path, root_path: &Path) -> String {
    file_path
        .iter()
        .skip(root_path.iter().count())
        .collect::<PathBuf>()
        .with_extension("")
        .iter()
        .map(|segment| segment.to_str().unwrap_or("<invalid>"))
        .join(".")
}
