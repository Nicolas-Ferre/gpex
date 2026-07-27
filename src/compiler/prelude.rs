use crate::utils::indexing::NodeRef;
use crate::utils::reading::ReadFile;
use std::path::PathBuf;

pub(crate) const PRELUDE_FILE_COUNT: usize = 3;
pub(crate) const PRELUDE_TYPES_FILE_INDEX: usize = 0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreludeTypesEndLocation;

impl NodeRef for PreludeTypesEndLocation {
    fn file_index(&self) -> usize {
        PRELUDE_TYPES_FILE_INDEX
    }

    fn id(&self) -> u64 {
        u64::MAX
    }

    fn scope(&self) -> &[u64] {
        &[]
    }
}

pub(crate) fn files() -> [ReadFile; PRELUDE_FILE_COUNT] {
    [
        file(
            "types",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/prelude/types.gpex")),
        ),
        file(
            "type_operations",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/prelude/type_operations.gpex"
            )),
        ),
        file(
            "operators",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/prelude/operators.gpex"
            )),
        ),
    ]
}

pub(crate) fn is_prelude_file_index(file_index: usize) -> bool {
    file_index < PRELUDE_FILE_COUNT
}

fn file(name: &str, content: &str) -> ReadFile {
    ReadFile {
        content: content.into(),
        fs_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("prelude")
            .join(format!("{name}.gpex")),
        dot_path: format!("prelude.{name}"),
    }
}
