pub(crate) mod actions;
pub(crate) mod fns;
pub(crate) mod imports;
pub(crate) mod params;
pub(crate) mod types;
pub(crate) mod vars;

use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use actions::RepeatDefinition;
use imports::Import;

#[derive(Debug)]
pub(crate) enum Item {
    Import(Import),
    Var(VarDefinition),
    Const(ConstDefinition),
    Struct(StructDefinition),
    Fn(Box<FnDefinition>),
    Repeat(RepeatDefinition),
}

impl Item {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            &|context| Import::parse(context).map(Self::Import),
            &|context| VarDefinition::parse(context).map(Self::Var),
            &|context| ConstDefinition::parse(context).map(Self::Const),
            &|context| StructDefinition::parse(context).map(Self::Struct),
            &|context| FnDefinition::parse(context).map(Box::new).map(Self::Fn),
            &|context| RepeatDefinition::parse(context).map(Self::Repeat),
        ])
    }
}
