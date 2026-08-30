mod location;

pub use location::LogLocation;

use owo_colors::colors::{Blue, Green, Red, Yellow};
use owo_colors::{Color, OwoColorize, Stream};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::{fmt, io};

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
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}: {}", self.level, self.msg)?;
        if let Some(location) = &self.location {
            location.fmt(formatter, self.level)?;
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
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "├─ {}: {}", self.level, self.msg)?;
        if let Some(location) = &self.location {
            location.fmt(formatter, self.level)?;
        }
        Ok(())
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
    /// A hint.
    Hint,
}

impl Display for LogLevel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_colored(formatter, self.label())
    }
}

impl LogLevel {
    fn fmt_colored(self, formatter: &mut Formatter<'_>, string: &str) -> fmt::Result {
        match self {
            Self::Error => fmt_colored::<Red>(formatter, string),
            Self::Warning => fmt_colored::<Yellow>(formatter, string),
            Self::Info => fmt_colored::<Blue>(formatter, string),
            Self::Hint => fmt_colored::<Green>(formatter, string),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
        }
    }
}

fn fmt_colored<ColorType: Color>(formatter: &mut Formatter<'_>, string: &str) -> fmt::Result {
    write!(
        formatter,
        "{}",
        string.if_supports_color(Stream::Stderr, |string| string.fg::<ColorType>())
    )
}

fn fmt_italic(formatter: &mut Formatter<'_>, string: &str) -> fmt::Result {
    write!(
        formatter,
        "{}",
        string.if_supports_color(Stream::Stderr, |string| string.italic())
    )
}
