use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) enum CaseKind {
    Ok,
    Wgsl,
    Nok,
}

pub(crate) fn case_kind(path: &Path) -> Option<CaseKind> {
    let dir_name = crate::files::path_file_name(path);
    if dir_name.starts_with("ok_") {
        Some(CaseKind::Ok)
    } else if dir_name.starts_with("wgsl_") {
        Some(CaseKind::Wgsl)
    } else if dir_name.starts_with("nok_") {
        Some(CaseKind::Nok)
    } else {
        None
    }
}
