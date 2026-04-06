use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use std::ops::Range;
use std::path::Path;

impl SpanProps for ParseContext<'_> {
    fn slice(&self, span: Span) -> &str {
        &self.files[span.file_index].content[span.start..span.end]
    }

    fn fs_path(&self, span: Span) -> &Path {
        &self.files[span.file_index].fs_path
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

pub(crate) trait SpanProps {
    fn slice(&self, span: Span) -> &str;

    fn fs_path(&self, span: Span) -> &Path;
}
