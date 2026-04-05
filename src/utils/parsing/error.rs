use crate::utils::reading::ReadFile;
use crate::{Log, LogLevel, LogLocation};
use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct ParseError<'config> {
    pub(crate) file: &'config ReadFile,
    pub(crate) offset: usize,
    pub(crate) expected_tokens: Vec<&'static str>,
}

impl ParseError<'_> {
    pub(crate) fn merge(errors: &[Self]) -> Self {
        let max_offset = errors
            .iter()
            .map(|error| error.offset)
            .max()
            .unwrap_or_else(|| unreachable!("cannot merge empty array of errors"));
        Self {
            file: errors[0].file,
            offset: max_offset,
            expected_tokens: errors
                .iter()
                .filter(|error| error.offset == max_offset)
                .flat_map(|error| error.expected_tokens.iter())
                .copied()
                .collect(),
        }
    }

    pub(crate) fn to_error(&self) -> Log {
        let unique_tokens: Vec<_> = self.expected_tokens.iter().unique().collect();
        Log {
            level: LogLevel::Error,
            msg: "expected ".to_string()
                + &unique_tokens
                    .iter()
                    .enumerate()
                    .map(|(index, &expected)| {
                        if index == 0 {
                            expected.to_string()
                        } else if index == unique_tokens.len() - 1 {
                            format!(" or {expected}")
                        } else {
                            format!(", {expected}")
                        }
                    })
                    .collect::<String>(),
            location: Some(LogLocation {
                path: self.file.fs_path.clone(),
                code: self.file.content.clone(),
                span: self.offset..self.offset + 1,
            }),
            inner: vec![],
        }
    }
}
