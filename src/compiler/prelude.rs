use crate::utils::reading::ReadFile;
use std::path::PathBuf;

pub(crate) const PRELUDE_FILE_INDEX: usize = 0;

pub(crate) fn file() -> ReadFile {
    ReadFile {
        content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/prelude.gpex")).into(),
        fs_path: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/res/prelude.gpex")),
        dot_path: "prelude".into(),
    }
}
