use crate::compiletools::logs::{Log, LogLocation};
use crate::compiletools::parsing::Span;
use crate::compiletools::reading::ReadFile;

#[derive(Debug)]
pub(crate) struct ValidateCtx<'a> {
    pub(crate) logs: Vec<Log>,
    files: &'a [ReadFile],
}

impl<'a> ValidateCtx<'a> {
    pub(crate) fn new(files: &'a [ReadFile]) -> Self {
        Self {
            logs: vec![],
            files,
        }
    }

    pub(crate) fn loc(&self, span: &Span) -> LogLocation {
        let file = &self.files[span.file_index];
        LogLocation {
            path: file.path.clone(),
            code: file.content.clone(),
            span: span.range.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ValidateError;
