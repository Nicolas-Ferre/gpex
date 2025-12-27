use crate::compiletools::logs::{Log, LogLevel, LogLocation};
use crate::compiletools::reading::ReadFile;
use std::ops::Range;

pub(crate) type Parser<'a, T> = fn(&mut ParseCtx<'a>) -> Result<T, ParseError<'a>>;

#[derive(Debug, Clone)]
pub(crate) struct ParseCtx<'a> {
    pub(crate) file: &'a ReadFile,
    pub(crate) file_index: usize,
    pub(crate) files: &'a [ReadFile],
    offset: usize,
    next_id: u64,
    comment_prefix: &'a str,
    scope: Vec<u64>,
}

impl<'a> ParseCtx<'a> {
    pub(crate) fn new(
        file: &'a ReadFile,
        file_index: usize,
        files: &'a [ReadFile],
        next_id: u64,
        comment_prefix: &'a str,
    ) -> Self {
        Self {
            file,
            file_index,
            files,
            offset: 0,
            next_id,
            comment_prefix,
            scope: vec![],
        }
    }

    pub(crate) fn scope(&self) -> &[u64] {
        &self.scope
    }

    pub(crate) fn define_scope<O>(&mut self, mut f: impl FnMut(&mut Self, u64) -> O) -> O {
        let id = self.next_id();
        self.scope.push(id);
        let output = f(self, id);
        _ = self.scope.pop();
        output
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub(crate) fn parse_any<T>(&mut self, parsers: &[Parser<'a, T>]) -> Result<T, ParseError<'a>> {
        debug_assert!(!parsers.is_empty());
        let mut errors = vec![];
        let previous_ctx = self.clone();
        for parser in parsers {
            match parser(self) {
                Ok(node) => return Ok(node),
                Err(err) => {
                    errors.push(err);
                    self.clone_from(&previous_ctx);
                }
            }
        }
        Err(ParseError::merge(&errors))
    }

    pub(crate) fn parse_many<T>(
        &mut self,
        min: usize,
        max: usize,
        parser: Parser<'a, T>,
        separator_parser: Option<Parser<'a, ()>>,
    ) -> Result<(Vec<T>, Option<ParseError<'a>>), ParseError<'a>> {
        debug_assert!(min <= 1); // if removed, failing separator parsing should be better handled
        let mut items = vec![];
        let mut error = None;
        for i in 0..max {
            let previous = self.clone();
            if i > 0
                && let Some(sep) = separator_parser
                && sep(self).is_err()
            {
                *self = previous;
                break;
            }
            match parser(self) {
                Ok(parsed) => items.push(parsed),
                Err(err) => {
                    *self = previous;
                    if i < min {
                        return Err(err);
                    }
                    let tmp = &mut self.clone();
                    Span::parse_whitespaces_and_comments(tmp);
                    if !tmp.remaining_code().is_empty() {
                        error = Some(err);
                    }
                    break;
                }
            }
        }
        Ok((items, error))
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
pub(crate) struct ParseError<'a> {
    pub(crate) file: &'a ReadFile,
    pub(crate) offset: usize,
    pub(crate) expected: Vec<&'static str>,
}

impl ParseError<'_> {
    #[expect(clippy::expect_used)] // tests ensure this never occurs
    pub(crate) fn merge(errors: &[Self]) -> Self {
        debug_assert!(!errors.is_empty());
        let max_offset = errors
            .iter()
            .map(|err| err.offset)
            .max()
            .expect("internal error: cannot merge empty errors array");
        Self {
            file: errors[0].file,
            offset: max_offset,
            expected: errors
                .iter()
                .filter(|err| err.offset == max_offset)
                .flat_map(|err| err.expected.iter())
                .copied()
                .collect(),
        }
    }

    pub(crate) fn to_error(&self) -> Log {
        Log {
            level: LogLevel::Error,
            msg: "expected ".to_string()
                + &self
                    .expected
                    .iter()
                    .enumerate()
                    .map(|(index, &expected)| {
                        if index == 0 {
                            expected.to_string()
                        } else if index == self.expected.len() - 1 {
                            format!(" or {expected}")
                        } else {
                            format!(", {expected}")
                        }
                    })
                    .collect::<String>(),
            loc: Some(LogLocation {
                path: self.file.path.clone(),
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
    pub(crate) fn parse_symbol<'a>(
        ctx: &mut ParseCtx<'a>,
        symbol: Symbol,
    ) -> Result<Self, ParseError<'a>> {
        Self::parse_whitespaces_and_comments(ctx);
        if ctx.remaining_code().starts_with(symbol.slice) {
            let range = ctx.offset..ctx.offset + symbol.slice.len();
            let is_keyword = symbol.slice.chars().all(Self::is_keyword_char);
            let is_next_char_keyword = Self::is_next_char_keyword(ctx, range.clone());
            if !is_keyword || !is_next_char_keyword {
                ctx.offset = range.end;
                return Ok(Self {
                    file_index: ctx.file_index,
                    start: range.start,
                    end: range.end,
                    slice: ctx.file.content[range].into(),
                });
            }
        }
        Err(ParseError {
            file: ctx.file,
            offset: ctx.offset,
            expected: vec![symbol.name],
        })
    }

    pub(crate) fn parse_pattern<'a>(
        ctx: &mut ParseCtx<'a>,
        pattern: Pattern,
    ) -> Result<Self, ParseError<'a>> {
        Self::parse_whitespaces_and_comments(ctx);
        let error = || ParseError {
            file: ctx.file,
            offset: ctx.offset,
            expected: vec![pattern.name],
        };
        let len = Self::pattern_length(ctx, pattern).map_err(|()| error())?;
        let range = ctx.offset..ctx.offset + len;
        let is_excluded_token = pattern
            .excluded_tokens
            .contains(&&ctx.file.content[range.clone()]);
        if is_excluded_token || Self::is_next_char_keyword(ctx, range.clone()) {
            Err(error())
        } else {
            ctx.offset = range.end;
            Ok(Self {
                file_index: ctx.file_index,
                start: range.start,
                end: range.end,
                slice: ctx.file.content[range].into(),
            })
        }
    }

    fn pattern_length(ctx: &ParseCtx<'_>, pattern: Pattern) -> Result<usize, ()> {
        let mut len = 0;
        for part in pattern.parts {
            let code = ctx.code_from(ctx.offset + len);
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

    fn parse_whitespaces_and_comments(ctx: &mut ParseCtx<'_>) {
        loop {
            if ctx.remaining_code().starts_with(ctx.comment_prefix) {
                let code = ctx.remaining_code();
                let next_break_line_pos = code.find('\n').unwrap_or(code.len());
                ctx.offset += next_break_line_pos;
            }
            Self::parse_whitespaces(ctx);
            if !ctx.remaining_code().starts_with(ctx.comment_prefix) {
                break;
            }
        }
    }

    fn parse_whitespaces(ctx: &mut ParseCtx<'_>) {
        let trimmed_code = ctx.remaining_code().trim_start();
        ctx.offset += ctx.remaining_code().len() - trimmed_code.len();
    }

    fn is_keyword_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn is_next_char_keyword(ctx: &ParseCtx<'_>, range: Range<usize>) -> bool {
        ctx.code_from(range.end)
            .chars()
            .next()
            .is_some_and(Self::is_keyword_char)
    }
}
