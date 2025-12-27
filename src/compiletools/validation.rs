use crate::compiletools::logs::{Log, LogLocation};
use crate::compiletools::parsing::Span;
use crate::compiletools::reading::ReadFile;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct ValidateCtx<'a> {
    pub(crate) logs: Vec<Log>,
    pub(crate) root_path: &'a Path,
    files: &'a [ReadFile],
}

impl<'a> ValidateCtx<'a> {
    pub(crate) fn new(files: &'a [ReadFile], root_path: &'a Path) -> Self {
        Self {
            logs: vec![],
            root_path,
            files,
        }
    }

    pub(crate) fn loc(&self, span: &Span) -> LogLocation {
        let file = &self.files[span.file_index];
        LogLocation {
            path: file.path.clone(),
            span: span.start..span.end,
            code: file.content.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ValidateError;
