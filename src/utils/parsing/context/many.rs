use super::{ParseContext, Parser, ParserResult};
use crate::utils::parsing::error::ParseError;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SeparatorParser<'context> {
    None,
    NotTrailing(Parser<'context, ()>),
    MaybeTrailing(Parser<'context, ()>),
}

impl<'context> SeparatorParser<'context> {
    pub(super) fn parser(self) -> Option<Parser<'context, ()>> {
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

pub(super) fn parse<'config, T>(
    context: &mut ParseContext<'config>,
    item_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, T>,
    separator_parser: SeparatorParser<'config>,
    stop_excluded_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, ()>,
) -> Result<Vec<T>, ParseError<'config>> {
    let initial_context = context.clone();
    let first_item = match parse_first_item(context, &item_parser, &stop_excluded_parser) {
        ParseManyStepResult::Item(item) => item,
        ParseManyStepResult::End => return Ok(vec![]),
        ParseManyStepResult::Error(error) => return restore_error(context, initial_context, error),
    };
    parse_remaining_items(
        context,
        &item_parser,
        separator_parser,
        &stop_excluded_parser,
        &initial_context,
        vec![first_item],
    )
}

fn parse_remaining_items<'config, T>(
    context: &mut ParseContext<'config>,
    item_parser: &impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, T>,
    separator_parser: SeparatorParser<'config>,
    stop_excluded_parser: &impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, ()>,
    initial_context: &ParseContext<'config>,
    mut items: Vec<T>,
) -> Result<Vec<T>, ParseError<'config>> {
    loop {
        match parse_separator(context, separator_parser, stop_excluded_parser) {
            ParseManyStepResult::Item(()) => {}
            ParseManyStepResult::End => return Ok(items),
            ParseManyStepResult::Error(error) => {
                return restore_error(context, initial_context.clone(), error);
            }
        }
        match parse_many_next_item(context, item_parser, separator_parser, stop_excluded_parser) {
            ParseManyStepResult::Item(item) => items.push(item),
            ParseManyStepResult::End => return Ok(items),
            ParseManyStepResult::Error(error) => {
                return restore_error(context, initial_context.clone(), error);
            }
        }
    }
}

fn parse_many_next_item<'config, T>(
    context: &mut ParseContext<'config>,
    item_parser: &impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, T>,
    separator_parser: SeparatorParser<'config>,
    stop_excluded_parser: &impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, ()>,
) -> ParseManyStepResult<'config, T> {
    let result = parse_next_item(context, item_parser, separator_parser, stop_excluded_parser);
    if matches!(result, ParseManyStepResult::Error(_))
        && matches!(separator_parser, SeparatorParser::MaybeTrailing(_))
        && stop_excluded_parser(&mut context.clone()).is_ok()
    {
        ParseManyStepResult::End
    } else {
        result
    }
}

fn restore_error<'config, T>(
    context: &mut ParseContext<'config>,
    initial_context: ParseContext<'config>,
    error: ParseError<'config>,
) -> Result<T, ParseError<'config>> {
    *context = initial_context;
    Err(error)
}

fn parse_first_item<'config, T>(
    context: &mut ParseContext<'config>,
    item_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, T>,
    stop_excluded_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, ()>,
) -> ParseManyStepResult<'config, T> {
    let previous_context = context.clone();
    let item_error = match item_parser(context) {
        Ok(item) => return ParseManyStepResult::Item(item),
        Err(item_error) => item_error,
    };
    *context = previous_context;
    match stop_excluded_parser(&mut context.clone()) {
        Ok(()) => ParseManyStepResult::End,
        Err(stop_error) => ParseManyStepResult::Error(ParseError::merge(&[item_error, stop_error])),
    }
}

fn parse_separator<'config>(
    context: &mut ParseContext<'config>,
    separator_parser: SeparatorParser<'config>,
    stop_excluded_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, ()>,
) -> ParseManyStepResult<'config, ()> {
    let Some(separator_parser) = separator_parser.parser() else {
        return ParseManyStepResult::Item(());
    };
    let previous_context = context.clone();
    let Err(separator_error) = separator_parser(context) else {
        return ParseManyStepResult::Item(());
    };
    *context = previous_context;
    if let Err(stop_error) = stop_excluded_parser(&mut context.clone()) {
        ParseManyStepResult::Error(ParseError::merge(&[separator_error, stop_error]))
    } else {
        ParseManyStepResult::End
    }
}

fn parse_next_item<'config, T>(
    context: &mut ParseContext<'config>,
    item_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, T>,
    separator_parser: SeparatorParser<'config>,
    stop_excluded_parser: impl Fn(&mut ParseContext<'config>) -> ParserResult<'config, ()>,
) -> ParseManyStepResult<'config, T> {
    let previous_context = context.clone();
    let item_error = match item_parser(context) {
        Ok(item) => return ParseManyStepResult::Item(item),
        Err(error) => error,
    };
    *context = previous_context;
    match stop_excluded_parser(&mut context.clone()) {
        Ok(()) if separator_parser.parser().is_some() => ParseManyStepResult::Error(item_error),
        Ok(()) => ParseManyStepResult::End,
        Err(_) => ParseManyStepResult::Error(item_error),
    }
}
