use crate::utils::logs::{Log, LogLevel, LogLocation};
use crate::utils::reading::ReadFile;
use std::ops::Range;

pub(crate) type Parser<'context, T> =
    fn(&mut ParseContext<'context>) -> Result<T, ParseError<'context>>;

#[derive(Debug, Clone)]
pub(crate) struct ParseContext<'config> {
    pub(crate) file: &'config ReadFile,
    pub(crate) file_index: usize,
    pub(crate) files: &'config [ReadFile],
    offset: usize,
    scope: Vec<u64>,
    next_id: u64,
    comment_prefix: &'config str,
}

impl<'config> ParseContext<'config> {
    pub(crate) fn new(
        file: &'config ReadFile,
        file_index: usize,
        files: &'config [ReadFile],
        next_id: u64,
        comment_prefix: &'config str,
    ) -> Self {
        Self {
            file,
            file_index,
            files,
            offset: 0,
            scope: vec![],
            next_id,
            comment_prefix,
        }
    }

    pub(crate) fn scope(&self) -> &[u64] {
        &self.scope
    }

    pub(crate) fn define_scope<O>(&mut self, mut scoped: impl FnMut(&mut Self, u64) -> O) -> O {
        let id = self.next_id();
        self.scope.push(id);
        let output = scoped(self, id);
        _ = self.scope.pop();
        output
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(crate) fn parse_any<T>(
        &mut self,
        parsers: &[Parser<'config, T>],
    ) -> Result<T, ParseError<'config>> {
        debug_assert!(!parsers.is_empty());
        let mut errors = vec![];
        let previous_context = self.clone();
        for parser in parsers {
            match parser(self) {
                Ok(node) => return Ok(node),
                Err(error) => {
                    errors.push(error);
                    self.clone_from(&previous_context);
                }
            }
        }
        Err(ParseError::merge(&errors))
    }

    pub(crate) fn parse_many<T>(
        &mut self,
        min: usize,
        item_parser: Parser<'config, T>,
        separator_parser: Option<Parser<'config, ()>>,
    ) -> Result<(Vec<T>, Option<ParseError<'config>>), ParseError<'config>> {
        debug_assert!(min <= 1); // if removed, failing separator parsing should be better handled
        let mut items = vec![];
        let mut item_index = 0;
        loop {
            Span::parse_whitespaces_and_comments(self);
            if self.remaining_code().is_empty() {
                break Ok((items, None));
            }
            let previous_context = self.clone();
            if item_index > 0
                && let Some(separator) = separator_parser
                && let Err(error) = separator(self)
            {
                *self = previous_context;
                break Ok((items, Some(error)));
            }
            match item_parser(self) {
                Ok(parsed) => items.push(parsed),
                Err(error) => {
                    *self = previous_context;
                    break if item_index < min {
                        Err(error)
                    } else {
                        Ok((items, Some(error)))
                    };
                }
            }
            item_index += 1;
        }
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

#[derive(Debug)]
pub(crate) struct ParseError<'config> {
    pub(crate) file: &'config ReadFile,
    pub(crate) offset: usize,
    pub(crate) expected_tokens: Vec<&'static str>,
}

impl ParseError<'_> {
    #[expect(clippy::unwrap_used)] // tests ensure this never occurs
    pub(crate) fn merge(errors: &[Self]) -> Self {
        debug_assert!(!errors.is_empty());
        let max_offset = errors.iter().map(|error| error.offset).max().unwrap();
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
        Log {
            level: LogLevel::Error,
            message: "expected ".to_string()
                + &self
                    .expected_tokens
                    .iter()
                    .enumerate()
                    .map(|(index, &expected)| {
                        if index == 0 {
                            expected.to_string()
                        } else if index == self.expected_tokens.len() - 1 {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Span {
    pub(crate) file_index: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) slice: String,
}

impl Span {
    pub(crate) fn until(&self, end: &Self) -> Self {
        Self {
            file_index: self.file_index,
            start: self.start,
            end: end.end,
            slice: String::new(),
        }
    }

    pub(crate) fn parse_symbol<'context>(
        context: &mut ParseContext<'context>,
        symbol: Symbol,
    ) -> Result<Self, ParseError<'context>> {
        Self::parse_whitespaces_and_comments(context);
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
                    slice: context.file.content[range].into(),
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
        Self::parse_whitespaces_and_comments(context);
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
                slice: context.file.content[range].into(),
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

    fn parse_whitespaces_and_comments(context: &mut ParseContext<'_>) {
        loop {
            if context.remaining_code().starts_with(context.comment_prefix) {
                let code = context.remaining_code();
                let next_break_line_offset = code.find('\n').unwrap_or(code.len());
                context.offset += next_break_line_offset;
            }
            Self::parse_whitespaces(context);
            if !context.remaining_code().starts_with(context.comment_prefix) {
                break;
            }
        }
    }

    fn parse_whitespaces(context: &mut ParseContext<'_>) {
        let trimmed_code = context.remaining_code().trim_start();
        context.offset += context.remaining_code().len() - trimmed_code.len();
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
