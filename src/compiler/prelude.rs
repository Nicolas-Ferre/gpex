use crate::utils::indexing::NodeRef;
use crate::utils::reading::ReadFile;
use std::path::PathBuf;

pub(crate) const PRELUDE_FILE_INDEX: usize = 0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreludeEndLocation;

impl NodeRef for PreludeEndLocation {
    fn file_index(&self) -> usize {
        PRELUDE_FILE_INDEX
    }

    fn id(&self) -> u64 {
        u64::MAX
    }

    fn scope(&self) -> &[u64] {
        &[]
    }
}

pub(crate) fn file() -> ReadFile {
    ReadFile {
        content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/prelude.gpex")).into(),
        fs_path: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/res/prelude.gpex")),
        dot_path: "prelude".into(),
    }
}
