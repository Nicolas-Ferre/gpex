use crate::compiler::parsing::items;
use crate::compiler::transversal::ast::modules::Module;
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Module, ParseError<'context>> {
    let items = context.parse_many(
        items::parse,
        SeparatorParser::None,
        ParseContext::parse_end_of_file,
    )?;
    Ok(Module {
        items,
        file_index: context.file_index,
    })
}
