pub(crate) mod actions;
pub(crate) mod fns;
pub(crate) mod imports;
pub(crate) mod params;
pub(crate) mod types;
pub(crate) mod vars;

use crate::compiler::transversal::ast::items::Item;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Item, ParseError<'context>> {
    context.parse_any(&[
        &|context| imports::parse(context).map(Item::Import),
        &|context| vars::parse_var_definition(context).map(Item::Var),
        &|context| vars::parse_const_definition(context).map(Item::Const),
        &|context| types::parse(context).map(Item::Struct),
        &|context| fns::parse(context).map(Box::new).map(Item::Fn),
        &|context| actions::parse(context).map(Item::Repeat),
    ])
}
