use itertools::Itertools;
use owo_colors::colors::xterm::LightGray;
use std::fmt::Formatter;
use std::ops::Range;
use std::path::PathBuf;

use super::{LogLevel, fmt_colored, fmt_italic};

const TAB_WIDTH: usize = 4;

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

impl LogLocation {
    pub(super) fn fmt(&self, formatter: &mut Formatter<'_>, level: LogLevel) -> std::fmt::Result {
        let start = self.line_column(self.span.start);
        let end = self.line_column(self.span.end);
        let rendered_lines = self.rendered_lines(start, end);
        let location = format!("{}:{}:{}\n", self.path.display(), start.line, start.column);
        write!(formatter, "│  → ")?;
        fmt_italic(formatter, &location)?;
        for (line_number, line) in rendered_lines {
            let displayed_line = Self::displayed_line(line);
            let span_spaces = " ".repeat(Self::line_span_offset(start, line_number, line));
            let span_underline = "^".repeat(Self::line_span_len(start, end, line_number, line));
            write!(formatter, "│    ")?;
            fmt_colored::<LightGray>(formatter, &format!("¦ {displayed_line}\n"))?;
            write!(formatter, "│    ")?;
            fmt_colored::<LightGray>(formatter, "¦ ")?;
            level.fmt_colored(formatter, &format!("{span_spaces}{span_underline}\n"))?;
        }
        Ok(())
    }

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

    fn line_span_offset(start: LocationCoords, line_number: usize, line: &str) -> usize {
        if line_number == start.line {
            Self::displayed_span_len(line, 1, start.column)
        } else {
            0
        }
    }

    fn line_span_len(
        start: LocationCoords,
        end: LocationCoords,
        line_number: usize,
        line: &str,
    ) -> usize {
        if line_number == start.line {
            if start.line == end.line {
                Self::displayed_span_len(line, start.column, end.column).max(1)
            } else {
                Self::displayed_span_len(line, start.column, line.chars().count() + 1)
            }
        } else if line_number == end.line {
            Self::displayed_span_len(line, 0, end.column)
        } else {
            Self::displayed_span_len(line, 0, line.chars().count() + 1)
        }
    }

    fn displayed_line(line: &str) -> String {
        let mut line_offset = 0;
        line.chars()
            .map(|char| {
                if char == '\t' {
                    let tab_size = TAB_WIDTH - line_offset % TAB_WIDTH;
                    line_offset += tab_size;
                    " ".repeat(tab_size)
                } else {
                    line_offset += 1;
                    char.to_string()
                }
            })
            .join("")
    }

    fn displayed_span_len(line: &str, column_start: usize, column_end: usize) -> usize {
        let mut line_offset = 0;
        let mut span_len = 0;
        for (index, char) in line.chars().enumerate() {
            let column = index + 1;
            let char_len = if char == '\t' {
                TAB_WIDTH - line_offset % TAB_WIDTH
            } else {
                1
            };
            line_offset += char_len;
            if column >= column_start && column < column_end {
                span_len += char_len;
            }
        }
        span_len
    }
}

#[derive(Debug, Clone, Copy)]
struct LocationCoords {
    line: usize,
    column: usize,
}
