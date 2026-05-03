use crate::utils::parsing::error::ParseError;
use crate::utils::reading::ReadFile;
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
        separator_parser: SeparatorParser<'config>,
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
                    separator_parser.parser().is_some(),
                    item_parser,
                    stop_excluded_parser,
                ) {
                    ParseManyStepResult::Item(item) => item,
                    ParseManyStepResult::End => break Ok(items),
                    ParseManyStepResult::Error(error) => {
                        if matches!(separator_parser, SeparatorParser::MaybeTrailing(_))
                            && stop_excluded_parser(&mut self.clone()).is_ok()
                        {
                            break Ok(items);
                        }
                        *self = initial_context;
                        break Err(error);
                    }
                },
            );
        }
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
        separator_parser: SeparatorParser<'config>,
        stop_excluded_parser: Parser<'config, ()>,
    ) -> ParseManyStepResult<'config, ()> {
        let Some(separator_parser) = separator_parser.parser() else {
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

    fn parse_whitespaces(&mut self) {
        let trimmed_code = self.remaining_code().trim_start(); // no-fn-check (recursivity)
        self.offset += self.remaining_code().len() - trimmed_code.len(); // no-fn-check (recursivity)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SeparatorParser<'context> {
    None,
    NotTrailing(Parser<'context, ()>),
    MaybeTrailing(Parser<'context, ()>),
}

impl<'context> SeparatorParser<'context> {
    fn parser(self) -> Option<Parser<'context, ()>> {
        match self {
            SeparatorParser::None => None,
            SeparatorParser::NotTrailing(parser) | SeparatorParser::MaybeTrailing(parser) => {
                Some(parser)
            }
        }
    }
}

enum ParseManyStepResult<'config, T> {
    Item(T),
    End,
    Error(ParseError<'config>),
}
