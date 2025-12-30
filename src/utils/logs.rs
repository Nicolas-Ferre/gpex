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
    pub message: String,
    /// A reference to the source code.
    pub location: Option<LogLocation>,
    /// Inner logs.
    pub inner: Vec<LogInner>,
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}: {}{}",
            self.level,
            self.message,
            if let Some(loc) = &self.location {
                format!(" (at {loc})")
            } else {
                String::new()
            }
        )?; // no-coverage (difficult to test)
        for inner in &self.inner {
            write!(f, "{inner}")?;
        }
        Ok(())
    }
}

impl Log {
    pub(crate) fn from_io_error(error: io::Error, path: &Path, message_prefix: &str) -> Self {
        Self {
            level: LogLevel::Error,
            message: format!("{} \"{}\": {}", message_prefix, path.display(), error),
            location: None,
            inner: vec![],
        }
    }
}

/// A compilation inner log.
#[derive(Debug)]
pub struct LogInner {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub message: String,
    /// A reference to the source code.
    pub location: Option<LogLocation>,
}

impl Display for LogInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "  --> {}: {}{}",
            self.level,
            self.message,
            if let Some(loc) = &self.location {
                format!(" (at {loc})")
            } else {
                String::new()
            }
        )
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
        write!(f, "{}:{line}:{column}", self.path.display())
    }
}

/// The level of a compilation log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// An information.
    Info,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}
