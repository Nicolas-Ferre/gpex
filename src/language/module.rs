use crate::compiler::indexes::Indexes;
use crate::language::import::Import;
use crate::language::items::constant::ConstantDefinition;
use crate::language::items::function::FunctionDefinition;
use crate::language::items::repeat::RepeatDefinition;
use crate::language::items::struct_::StructDefinition;
use crate::language::items::variable::VariableDefinition;
use crate::utils::parsing::{ParseContext, ParseError};
use crate::utils::validation::{ValidateContext, ValidateError};

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

    pub(crate) fn index_imports<'index>(&'index self, indexes: &mut Indexes<'index>) {
        for item in &self.items {
            item.index_imports(indexes);
        }
    }

    pub(crate) fn index_items<'index>(&'index self, indexes: &mut Indexes<'index>) {
        for item in &self.items {
            item.index_items(indexes);
        }
    }

    pub(crate) fn index_signatures(&self, indexes: &mut Indexes<'_>) {
        for item in &self.items {
            item.index_signatures(indexes);
        }
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        for item in &self.items {
            item.index_refs(indexes);
        }
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let mut is_module_valid = true;
        let mut are_imports_finished = false;
        for item in &self.items {
            if let Item::Import(import) = item {
                if import
                    .validate(!are_imports_finished, context, indexes)
                    .is_err()
                {
                    is_module_valid = false;
                }
            } else {
                are_imports_finished = true;
            }
        }
        if !is_module_valid {
            return Err(ValidateError);
        }
        for item in &self.items {
            _ = item.validate(context, indexes);
        }
        Ok(())
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

#[derive(Debug)]
pub(crate) enum Item {
    Import(Import),
    Variable(VariableDefinition),
    Constant(ConstantDefinition),
    Struct(StructDefinition),
    Function(FunctionDefinition),
    Repeat(RepeatDefinition),
}

impl Item {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.parse_any(&[
            |context| Import::parse(context).map(Self::Import),
            |context| VariableDefinition::parse(context).map(Self::Variable),
            |context| ConstantDefinition::parse(context).map(Self::Constant),
            |context| StructDefinition::parse(context).map(Self::Struct),
            |context| FunctionDefinition::parse(context).map(Self::Function),
            |context| RepeatDefinition::parse(context).map(Self::Repeat),
        ])
    }

    pub(crate) fn index_imports<'index>(&'index self, indexes: &mut Indexes<'index>) {
        match self {
            Self::Import(item) => item.index(indexes),
            Self::Variable(_)
            | Self::Constant(_)
            | Self::Struct(_)
            | Self::Function(_)
            | Self::Repeat(_) => (),
        }
    }

    pub(crate) fn index_items<'index>(&'index self, indexes: &mut Indexes<'index>) {
        match self {
            Self::Import(_) | Self::Repeat(_) => (),
            Self::Variable(item) => item.index_item(indexes),
            Self::Constant(item) => item.index_item(indexes),
            Self::Struct(item) => item.index_item(indexes),
            Self::Function(item) => item.index_item(indexes),
        }
    }

    pub(crate) fn index_signatures(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Function(item) => item.index_signatures(indexes),
            Self::Import(_)
            | Self::Variable(_)
            | Self::Constant(_)
            | Self::Struct(_)
            | Self::Repeat(_) => (),
        }
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Variable(item) => item.index_refs(indexes),
            Self::Constant(item) => item.index_refs(indexes),
            Self::Function(item) => item.index_refs(indexes),
            Self::Repeat(item) => item.index_refs(indexes),
            Self::Import(_) | Self::Struct(_) => (),
        }
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::Import(_) => Ok(()), // validated during previous pass
            Self::Variable(item) => item.validate(context, indexes),
            Self::Constant(item) => item.validate(context, indexes),
            Self::Struct(item) => item.validate(context),
            Self::Function(item) => item.validate(context, indexes),
            Self::Repeat(item) => item.validate(context, indexes),
        }
    }
}
