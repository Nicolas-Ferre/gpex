use crate::compiler::parsing::items::Item;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Module {
    #[derive_where(skip)]
    pub(crate) items: Vec<Item>,
    pub(crate) file_index: usize,
}

impl Module {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let items = context.parse_many(Item::parse, None, ParseContext::parse_end_of_file)?;
        Ok(Self {
            items,
            file_index: context.file_index,
        })
    }

    pub(crate) fn global_vars(&self) -> impl Iterator<Item = &VarDefinition> {
        self.items.iter().filter_map(|item| {
            if let Item::Var(var) = item {
                Some(var)
            } else {
                None
            }
        })
    }

    pub(crate) fn repeats(&self) -> impl Iterator<Item = &RepeatDefinition> {
        self.items.iter().filter_map(|item| {
            if let Item::Repeat(repeat) = item {
                Some(repeat)
            } else {
                None
            }
        })
    }
}
