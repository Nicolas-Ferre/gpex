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
            writeln!(formatter, "│  → {location}")?;
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
            writeln!(formatter, "│  → {location}")?;
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
        let mut line = 1;
        let mut column = 1;
        for (offset, char) in self.code.char_indices() {
            if offset == self.span.start {
                break;
            } else if char == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        write!(formatter, "{}:{line}:{column}", self.path.display())
    }
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
