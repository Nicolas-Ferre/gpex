use crate::compiler::indexes::Indexes;
use crate::language::import::Import;
use crate::language::items::constant::ConstantDefinition;
use crate::language::items::function::FunctionDefinition;
use crate::language::items::struct_::StructDefinition;
use crate::language::items::variable::VariableDefinition;
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

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
        let (items, error) = context.parse_many(0, Item::parse, None)?;
        if let Some(error) = error {
            return Err(error);
        }
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
        let dependencies = Dependencies::new();
        let dependencies = Self::collect_dependencies(dependencies, self.file_index, indexes);
        validators::import::check_circular_dependencies(dependencies, context)?;
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
            return Err(ValidateError);
        }
        for item in &self.items {
            _ = item.validate(context, indexes);
        }
        Ok(())
    }

    fn collect_dependencies(
        mut dependencies: Dependencies<usize>,
        file_index: usize,
        indexes: &Indexes<'_>,
    ) -> Result<Dependencies<usize>, Vec<Span>> {
        for import in indexes.imports.imports(file_index) {
            if let Some(span) = import.span {
                dependencies = dependencies.register(span, import.file_index, |dependencies| {
                    Self::collect_dependencies(dependencies, import.file_index, indexes)
                })?;
            }
        }
        Ok(dependencies)
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
    Struct(StructDefinition),
    Function(FunctionDefinition),
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
        ])
    }

    pub(crate) fn index_imports<'index>(&'index self, indexes: &mut Indexes<'index>) {
        match self {
            Self::Import(item) => item.index(indexes),
            Self::Variable(_) | Self::Constant(_) | Self::Struct(_) | Self::Function(_) => (),
        }
    }

    pub(crate) fn index_items<'index>(&'index self, indexes: &mut Indexes<'index>) {
        match self {
            Self::Import(_) => (),
            Self::Variable(item) => item.index_item(indexes),
            Self::Constant(item) => item.index_item(indexes),
            Self::Struct(item) => item.index_item(indexes),
            Self::Function(item) => item.index_item(indexes),
        }
    }

    pub(crate) fn index_refs(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Variable(item) => item.index_refs(indexes),
            Self::Constant(item) => item.index_refs(indexes),
            Self::Function(item) => item.index_refs(indexes),
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
        }
    }
}
