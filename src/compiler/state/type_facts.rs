use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::types::Type;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub(crate) type TypeFactContext<'item> = HashMap<TypeFactSubject<'item>, Rc<TypeFacts<'item>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TypeFactSubject<'item> {
    Wildcard(&'item Param),
    Referenced(&'item Param),
}

impl<'item> TypeFactSubject<'item> {
    pub(crate) fn from_type(type_: Type<'item>) -> Option<Self> {
        match type_ {
            Type::Param(param) => Some(Self::Referenced(param)),
            Type::Wildcard(param) => Some(Self::Wildcard(param)),
            Type::Struct(_) | Type::NoReturn | Type::Unknown => None,
        }
    }

    pub(crate) fn from_param_type(param: &'item Param, type_: Type<'item>) -> Self {
        match type_ {
            Type::Param(type_param) => Self::Referenced(type_param),
            Type::Wildcard(type_param) => Self::Wildcard(type_param),
            Type::NoReturn | Type::Unknown => Self::Wildcard(param),
            Type::Struct(_) => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TypeFacts<'item> {
    required_types: HashSet<&'item StructDefinition>,
}

impl<'item> TypeFacts<'item> {
    pub(crate) fn required_type(&self) -> Option<&'item StructDefinition> {
        (self.required_types.len() == 1)
            .then(|| self.required_types.iter().next().copied())
            .flatten()
    }

    pub(crate) fn has_contradiction(&self) -> bool {
        self.required_types.len() > 1
    }

    pub(crate) fn add_required_type(&mut self, type_: &'item StructDefinition) {
        self.required_types.insert(type_);
    }

    pub(crate) fn add_required_types(&mut self, other: &Self) {
        self.required_types.extend(&other.required_types);
    }
}
