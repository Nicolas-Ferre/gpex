use crate::compiler::indexes::Indexes;
use crate::language::import::Import;
use crate::language::items::const_::ConstantDefinition;
use crate::language::items::var::VariableDefinition;
use crate::utils::parsing::{ParseContext, ParseError};
use crate::utils::validation::{ValidateContext, ValidateError};

#[derive(Debug)]
pub(crate) struct Module {
    pub(crate) items: Vec<Item>,
}

impl Module {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let (items, error) = context.parse_many(0, Item::parse, None)?;
        if let Some(error) = error {
            return Err(error);
        }
        Ok(Self { items })
    }

    pub(crate) fn index_items<'index>(&'index self, indexes: &mut Indexes<'index>) {
        for item in &self.items {
            item.index_items(indexes);
        }
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        for item in &self.items {
            item.index_refs(indexes);
        }
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>, indexes: &mut Indexes<'_>) {
        let mut is_module_invalid = false;
        let mut are_imports_finished = false;
        for item in &self.items {
            if let Item::Import(import) = item {
                if import
                    .validate(!are_imports_finished, context, indexes)
                    .is_err()
                {
                    is_module_invalid = true;
                }
            } else {
                are_imports_finished = true;
            }
        }
        if is_module_invalid {
            return;
        }
        for item in &self.items {
            _ = item.validate(context, indexes);
        }
    }

    pub(crate) fn global_variables(&self) -> impl Iterator<Item = &VariableDefinition> {
        self.items.iter().filter_map(|item| {
            if let Item::Variable(variable) = item {
                Some(variable)
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
pub(crate) enum Item {
    Import(Import),
    Variable(VariableDefinition),
    Constant(ConstantDefinition),
}

impl Item {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| Import::parse(context).map(Self::Import),
            |context| VariableDefinition::parse(context).map(Self::Variable),
            |context| ConstantDefinition::parse(context).map(Self::Constant),
        ])
    }

    pub(crate) fn index_items<'index>(&'index self, indexes: &mut Indexes<'index>) {
        match self {
            Self::Import(item) => item.index(indexes),
            Self::Variable(item) => item.index_item(indexes),
            Self::Constant(item) => item.index_item(indexes),
        }
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Import(_) => (),
            Self::Variable(item) => item.index_refs(indexes),
            Self::Constant(item) => item.index_refs(indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::Import(_) => Ok(()), // validated during previous pass
            Self::Variable(item) => item.validate(context, indexes),
            Self::Constant(item) => item.validate(context, indexes),
        }
    }
}
