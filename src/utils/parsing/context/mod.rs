mod many;

pub(crate) use many::SeparatorParser;

use crate::utils::parsing::error::ParseError;
use crate::utils::reading::ReadFile;
use std::path::Path;

pub(crate) type ParserResult<'context, T> = Result<T, ParseError<'context>>;
pub(crate) type Parser<'context, T> = fn(&mut ParseContext<'context>) -> ParserResult<'context, T>;

#[derive(Debug, Clone)]
pub(crate) struct ParseContext<'config> {
    pub(crate) root_path: &'config Path,
    pub(crate) file: &'config ReadFile,
    pub(crate) file_index: usize,
    pub(crate) files: &'config [ReadFile],
    is_parse_any_error_forced: bool,
    pub(super) offset: usize,
    scope: Vec<u64>,
    next_id: u64,
    comment_prefix: &'config str,
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

    pub(crate) fn define_scope<O>(&mut self, scoped: impl FnOnce(&mut Self, u64) -> O) -> O {
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

    #[expect(clippy::type_complexity)] // dyn parser closure is only used here
    pub(crate) fn parse_any<T>(
        &mut self,
        parsers: &[&dyn Fn(&mut Self) -> ParserResult<'config, T>],
    ) -> Result<T, ParseError<'config>> {
        debug_assert!(!parsers.is_empty());
        let mut errors = vec![];
        let initial_context = self.clone();
        for parser in parsers {
            self.is_parse_any_error_forced = false;
            match parser(self) {
                Ok(value) => {
                    self.is_parse_any_error_forced = initial_context.is_parse_any_error_forced;
                    return Ok(value);
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
        item_parser: impl Fn(&mut Self) -> ParserResult<'config, T>,
        separator_parser: SeparatorParser<'config>,
        stop_excluded_parser: impl Fn(&mut Self) -> ParserResult<'config, ()>,
    ) -> Result<Vec<T>, ParseError<'config>> {
        many::parse(self, item_parser, separator_parser, stop_excluded_parser)
    }

    pub(super) fn parse_whitespaces_and_comments(&mut self) {
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

    pub(super) fn remaining_code(&self) -> &str {
        self.code_from(self.offset)
    }

    pub(super) fn code_from(&self, offset: usize) -> &str {
        if offset >= self.file.content.len() {
            ""
        } else {
            &self.file.content[offset..]
        }
    }

    fn parse_whitespaces(&mut self) {
        let trimmed_code = self.remaining_code().trim_start();
        self.offset += self.remaining_code().len() - trimmed_code.len();
    }
}
