use crate::compiler::transversal::ast::items::Item;
use crate::compiler::transversal::ast::items::actions::RepeatDefinition;
use crate::compiler::transversal::ast::items::vars::VarDefinition;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Module {
    #[derive_where(skip)]
    pub(crate) items: Vec<Item>,
    pub(crate) file_index: usize,
}

impl Module {
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
