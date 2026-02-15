use crate::language::items::struct_::StructDefinition;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Type<'item> {
    Struct(&'item StructDefinition),
    NoReturn,
    Unknown,
}

impl<'item> Type<'item> {
    pub(crate) fn struct_ref(self) -> Option<&'item StructDefinition> {
        if let Self::Struct(struct_) = self {
            Some(struct_)
        } else {
            None
        }
    }
}
