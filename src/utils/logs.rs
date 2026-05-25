use std::fmt::{Display, Formatter};
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A compilation log.
#[derive(Debug)]
pub struct Log {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub msg: String,
    /// A reference to the source code.
    pub location: Option<LogLocation>,
    /// Inner logs.
    pub inner: Vec<LogInner>,
}

impl Display for Log {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "{}: {}", self.level, self.msg)?;
        if let Some(location) = &self.location {
            location.fmt(formatter)?;
        }
        for inner in &self.inner {
            write!(formatter, "{inner}")?;
        }
        if self.location.is_some() || !self.inner.is_empty() {
            writeln!(formatter, "┴")?;
        }
        Ok(())
    }
}

impl Log {
    pub(crate) fn from_io_error(error: io::Error, path: &Path, msg_prefix: &str) -> Self {
        Self {
            level: LogLevel::Error,
            msg: format!("{msg_prefix} \"{}\": {error}", path.display()),
            location: None,
            inner: vec![],
        }
    }

    pub(crate) fn sort_key(&self) -> (LogLevel, Option<(PathBuf, usize)>) {
        (
            self.level,
            self.location
                .as_ref()
                .map(|location| (location.path.clone(), location.span.start)),
        )
    }
}

/// A compilation inner log.
#[derive(Debug)]
pub struct LogInner {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub msg: String,
    /// A reference to the source code.
    pub location: Option<LogLocation>,
}

impl Display for LogInner {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "├─ {}: {}", self.level, self.msg)?;
        if let Some(location) = &self.location {
            location.fmt(formatter)?;
        }
        Ok(())
    }
}

/// A reference to the source code.
#[derive(Debug)]
pub struct LogLocation {
    /// The file path.
    pub path: PathBuf,
    /// The source code.
    pub code: String,
    /// The reference span.
    pub span: Range<usize>,
}

impl Display for LogLocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let start = self.line_column(self.span.start);
        let end = self.line_column(self.span.end);
        let rendered_lines = self.rendered_lines(start, end);
        write!(formatter, "│  → {}", self.path.display())?;
        writeln!(formatter, ":{}:{}", start.line, start.column)?;
        for (line_number, line) in rendered_lines {
            let span_spaces = " ".repeat(Self::line_span_offset(start, line_number));
            let span_underline = "^".repeat(self.line_span_len(start, end, line_number, line));
            writeln!(formatter, "│    ¦ {line}")?;
            writeln!(formatter, "│    ¦ {span_spaces}{span_underline}")?;
        }
        Ok(())
    }
}

impl LogLocation {
    fn line_column(&self, target_offset: usize) -> LocationCoords {
        let mut line = 1;
        let mut column = 1;
        for (offset, char) in self.code.char_indices() {
            if offset == target_offset {
                break;
            } else if char == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        LocationCoords { line, column }
    }

    fn rendered_lines(
        &self,
        start: LocationCoords,
        end: LocationCoords,
    ) -> impl Iterator<Item = (usize, &str)> {
        self.code
            .split('\n')
            .enumerate()
            .map(|(line_number, line)| (line_number + 1, line))
            .skip(start.line - 1)
            .take(end.line - start.line + 1)
    }

    fn line_span_offset(start: LocationCoords, line_number: usize) -> usize {
        if line_number == start.line {
            start.column - 1
        } else {
            0
        }
    }

    fn line_span_len(
        &self,
        start: LocationCoords,
        end: LocationCoords,
        line_number: usize,
        line: &str,
    ) -> usize {
        if line_number == start.line {
            if start.line == end.line {
                self.span.len()
            } else {
                line.chars().count() - start.column + 1
            }
        } else if line_number == end.line {
            end.column - 1
        } else {
            line.chars().count()
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LocationCoords {
    line: usize,
    column: usize,
}

/// The level of a compilation log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// An information.
    Info,
}

impl Display for LogLevel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(formatter, "error"),
            Self::Warning => write!(formatter, "warning"),
            Self::Info => write!(formatter, "info"),
        }
    }
}
