use crate::utils::logs::{Log, LogLocation};
use crate::utils::parsing::{Span, SpanProperties};
use crate::utils::reading::ReadFile;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct ValidateContext<'config> {
    pub(crate) logs: Vec<Log>,
    pub(crate) root_path: &'config Path,
    files: &'config [ReadFile],
}

impl SpanProperties for ValidateContext<'_> {
    fn slice(&self, span: Span) -> &str {
        &self.files[span.file_index].content[span.start..span.end]
    }

    fn fs_path(&self, span: Span) -> &Path {
        &self.files[span.file_index].fs_path
    }
}

impl<'config> ValidateContext<'config> {
    pub(crate) fn new(files: &'config [ReadFile], root_path: &'config Path) -> Self {
        Self {
            logs: vec![],
            root_path,
            files,
        }
    }

    pub(crate) fn location(&self, span: Span) -> LogLocation {
        let file = &self.files[span.file_index];
        LogLocation {
            path: file.fs_path.clone(),
            span: span.start..span.end,
            code: file.content.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ValidateError;
