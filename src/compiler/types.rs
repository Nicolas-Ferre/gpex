use crate::language::items::struct_::StructDefinition;
use derive_where::derive_where;

#[derive(Debug, Clone, Copy)]
#[derive_where(PartialEq)]
pub(crate) enum Type<'item> {
    Struct(&'item StructDefinition),
    NoReturn,
    #[derive_where(incomparable)]
    Unknown,
}

impl<'item> Type<'item> {
    pub(crate) fn name(self) -> &'item str {
        match self {
            Type::Struct(struct_) => &struct_.name,
            Type::NoReturn => unreachable!("no-type expression is not allowed as argument"),
            Type::Unknown => unreachable!("unknown-type expression is not allowed as argument"),
        }
    }

    pub(crate) fn struct_ref(self) -> Option<&'item StructDefinition> {
        if let Self::Struct(struct_) = self {
            Some(struct_)
        } else {
            None
        }
    }
}
