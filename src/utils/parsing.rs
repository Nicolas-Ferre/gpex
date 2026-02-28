use crate::utils::logs::{Log, LogLevel, LogLocation};
use crate::utils::reading::ReadFile;
use itertools::Itertools;
use std::ops::Range;
use std::path::Path;

pub(crate) type Parser<'context, T> =
    fn(&mut ParseContext<'context>) -> Result<T, ParseError<'context>>;

#[derive(Debug, Clone)]
pub(crate) struct ParseContext<'config> {
    pub(crate) root_path: &'config Path,
    pub(crate) file: &'config ReadFile,
    pub(crate) file_index: usize,
    pub(crate) files: &'config [ReadFile],
    is_parse_any_error_forced: bool,
    offset: usize,
    scope: Vec<u64>,
    next_id: u64,
    comment_prefix: &'config str,
}

impl SpanProperties for ParseContext<'_> {
    fn slice(&self, span: Span) -> &str {
        &self.files[span.file_index].content[span.start..span.end]
    }

    fn fs_path(&self, span: Span) -> &Path {
        &self.files[span.file_index].fs_path
    }
}

impl<'config> ParseContext<'config> {
    pub(crate) fn new(
        root_path: &'config Path,
        file: &'config ReadFile,
        file_index: usize,
        files: &'config [ReadFile],
        next_id: u64,
        comment_prefix: &'config str,
    ) -> Self {
        Self {
            root_path,
            file,
            file_index,
            files,
            is_parse_any_error_forced: false,
            offset: 0,
            scope: vec![],
            next_id,
            comment_prefix,
        }
    }

    /// Forces any parsing error in the current `parse_any()` branch
    /// instead of testing alternative branches.
    pub(crate) fn force_parse_any_error(&mut self) {
        self.is_parse_any_error_forced = true;
    }

    pub(crate) fn scope(&self) -> &[u64] {
        &self.scope
    }

    pub(crate) fn define_scope<O>(&mut self, mut scoped: impl FnMut(&mut Self, u64) -> O) -> O {
        let id = self.next_id();
        self.scope.push(id);
        let output = scoped(self, id);
        self.scope.pop();
        output
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(crate) fn parse_end_of_file(&mut self) -> Result<(), ParseError<'config>> {
        self.parse_whitespaces_and_comments();
        if self.remaining_code().is_empty() {
            Ok(())
        } else {
            Err(ParseError {
                file: self.file,
                offset: self.offset,
                expected_tokens: vec!["end of file"],
            })
        }
    }

    pub(crate) fn parse_any<T>(
        &mut self,
        parsers: &[Parser<'config, T>],
    ) -> Result<T, ParseError<'config>> {
        debug_assert!(!parsers.is_empty());
        let mut errors = vec![];
        let initial_context = self.clone();
        for parser in parsers {
            self.is_parse_any_error_forced = false;
            match parser(self) {
                Ok(node) => {
                    self.is_parse_any_error_forced = initial_context.is_parse_any_error_forced;
                    return Ok(node);
                }
                Err(error) if self.is_parse_any_error_forced => {
                    *self = initial_context;
                    return Err(error);
                }
                Err(error) => {
                    self.clone_from(&initial_context);
                    errors.push(error);
                }
            }
        }
        self.clone_from(&initial_context);
        Err(ParseError::merge(&errors))
    }

    pub(crate) fn parse_many<T>(
        &mut self,
        item_parser: Parser<'config, T>,
        separator_parser: Option<Parser<'config, ()>>,
        stop_excluded_parser: Parser<'config, ()>,
    ) -> Result<Vec<T>, ParseError<'config>> {
        let initial_context = self.clone();
        let mut items = vec![
            match self.parse_first_item(item_parser, stop_excluded_parser) {
                ParseManyStepResult::Item(item) => item,
                ParseManyStepResult::End => return Ok(vec![]),
                ParseManyStepResult::Error(error) => {
                    *self = initial_context;
                    return Err(error);
                }
            },
        ];
        loop {
            match self.parse_separator(separator_parser, stop_excluded_parser) {
                ParseManyStepResult::Item(()) => {}
                ParseManyStepResult::End => break Ok(items),
                ParseManyStepResult::Error(error) => {
                    *self = initial_context;
                    break Err(error);
                }
            }
            items.push(
                match self.parse_next_item(
                    separator_parser.is_some(),
                    item_parser,
                    stop_excluded_parser,
                ) {
                    ParseManyStepResult::Item(item) => item,
                    ParseManyStepResult::End => break Ok(items),
                    ParseManyStepResult::Error(error) => {
                        *self = initial_context;
                        break Err(error);
                    }
                },
            );
        }
    }

    fn parse_first_item<T>(
        &mut self,
        item_parser: Parser<'config, T>,
        stop_excluded_parser: Parser<'config, ()>,
    ) -> ParseManyStepResult<'config, T> {
        let previous_context = self.clone();
        let item_error = match item_parser(self) {
            Ok(item) => return ParseManyStepResult::Item(item),
            Err(item_error) => item_error,
        };
        *self = previous_context;
        match stop_excluded_parser(&mut self.clone()) {
            Ok(()) => ParseManyStepResult::End,
            Err(stop_error) => {
                ParseManyStepResult::Error(ParseError::merge(&[item_error, stop_error]))
            }
        }
    }

    fn parse_separator(
        &mut self,
        separator_parser: Option<Parser<'config, ()>>,
        stop_excluded_parser: Parser<'config, ()>,
    ) -> ParseManyStepResult<'config, ()> {
        let Some(separator_parser) = separator_parser else {
            return ParseManyStepResult::Item(());
        };
        let previous_context = self.clone();
        let Err(separator_error) = separator_parser(self) else {
            return ParseManyStepResult::Item(());
        };
        *self = previous_context;
        if let Err(stop_error) = stop_excluded_parser(&mut self.clone()) {
            ParseManyStepResult::Error(ParseError::merge(&[separator_error, stop_error]))
        } else {
            ParseManyStepResult::End
        }
    }

    fn parse_next_item<T>(
        &mut self,
        has_separator: bool,
        item_parser: Parser<'config, T>,
        stop_excluded_parser: Parser<'config, ()>,
    ) -> ParseManyStepResult<'config, T> {
        let previous_context = self.clone();
        let item_error = match item_parser(self) {
            Ok(item) => return ParseManyStepResult::Item(item),
            Err(error) => error,
        };
        *self = previous_context;
        match stop_excluded_parser(&mut self.clone()) {
            Ok(()) if has_separator => ParseManyStepResult::Error(item_error),
            Ok(()) => ParseManyStepResult::End,
            Err(_) => ParseManyStepResult::Error(item_error),
        }
    }

    fn parse_whitespaces_and_comments(&mut self) {
        loop {
            if self.remaining_code().starts_with(self.comment_prefix) {
                let code = self.remaining_code();
                let next_break_line_offset = code.find('\n').unwrap_or(code.len());
                self.offset += next_break_line_offset;
            }
            self.parse_whitespaces();
            if !self.remaining_code().starts_with(self.comment_prefix) {
                break;
            }
        }
    }

    fn parse_whitespaces(&mut self) {
        let trimmed_code = self.remaining_code().trim_start();
        self.offset += self.remaining_code().len() - trimmed_code.len();
    }

    fn remaining_code(&self) -> &str {
        self.code_from(self.offset)
    }

    fn code_from(&self, offset: usize) -> &str {
        if offset >= self.file.content.len() {
            ""
        } else {
            &self.file.content[offset..]
        }
    }
}

enum ParseManyStepResult<'config, T> {
    Item(T),
    End,
    Error(ParseError<'config>),
}

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
            message: "expected ".to_string()
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Symbol {
    pub(crate) name: &'static str,
    pub(crate) slice: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Pattern {
    pub(crate) name: &'static str,
    pub(crate) excluded_tokens: &'static [&'static str],
    pub(crate) parts: &'static [PatternPart],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PatternPart {
    pub(crate) is_valid_char: fn(char) -> bool,
    pub(crate) min_count: usize,
    pub(crate) max_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Span {
    pub(crate) file_index: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub(crate) fn until(&self, end: Self) -> Self {
        Self {
            file_index: self.file_index,
            start: self.start,
            end: end.end,
        }
    }

    pub(crate) fn parse_symbol<'context>(
        context: &mut ParseContext<'context>,
        symbol: Symbol,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_whitespaces_and_comments();
        if context.remaining_code().starts_with(symbol.slice) {
            let range = context.offset..context.offset + symbol.slice.len();
            let is_keyword = symbol.slice.chars().all(Self::is_char_keyword);
            let is_next_char_keyword = Self::is_next_char_keyword(context, range.clone());
            if !is_keyword || !is_next_char_keyword {
                context.offset = range.end;
                return Ok(Self {
                    file_index: context.file_index,
                    start: range.start,
                    end: range.end,
                });
            }
        }
        Err(ParseError {
            file: context.file,
            offset: context.offset,
            expected_tokens: vec![symbol.name],
        })
    }

    pub(crate) fn parse_pattern<'context>(
        context: &mut ParseContext<'context>,
        pattern: Pattern,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_whitespaces_and_comments();
        let error = || ParseError {
            file: context.file,
            offset: context.offset,
            expected_tokens: vec![pattern.name],
        };
        let len = Self::pattern_len(context, pattern).map_err(|()| error())?;
        let range = context.offset..context.offset + len;
        let is_token_excluded = pattern
            .excluded_tokens
            .contains(&&context.file.content[range.clone()]);
        if is_token_excluded || Self::is_next_char_keyword(context, range.clone()) {
            Err(error())
        } else {
            context.offset = range.end;
            Ok(Self {
                file_index: context.file_index,
                start: range.start,
                end: range.end,
            })
        }
    }

    fn pattern_len(context: &ParseContext<'_>, pattern: Pattern) -> Result<usize, ()> {
        let mut len = 0;
        for part in pattern.parts {
            let code = context.code_from(context.offset + len);
            if code.is_empty() && part.min_count > 0 {
                return Err(());
            }
            for (index, char) in code.chars().enumerate() {
                if index >= part.max_count {
                    break;
                } else if (part.is_valid_char)(char) {
                    len += char.len_utf8();
                } else if index >= part.min_count {
                    break;
                } else {
                    return Err(());
                }
            }
        }
        Ok(len)
    }

    fn is_char_keyword(char: char) -> bool {
        char.is_ascii_alphanumeric() || char == '_'
    }

    fn is_next_char_keyword(context: &ParseContext<'_>, range: Range<usize>) -> bool {
        context
            .code_from(range.end)
            .chars()
            .next()
            .is_some_and(Self::is_char_keyword)
    }
}

pub(crate) trait SpanProperties {
    fn slice(&self, span: Span) -> &str;

    fn fs_path(&self, span: Span) -> &Path;
}
