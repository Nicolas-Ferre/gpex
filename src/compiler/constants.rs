use crate::language::items::struct_::StructDefinition;
use crate::utils::formatting;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub(crate) enum Constant<'item> {
    TypeRef(&'item StructDefinition),
    I32(i32),
    U32(u32),
    F32(f32),
    Bool(bool),
    Unknown,
}

impl<'item> Constant<'item> {
    pub(crate) fn transpile(&self, shader: &mut String) {
        match self {
            Self::TypeRef(value) => value.transpile_ref(shader),
            Self::I32(value) => _ = write!(shader, "i32({value})"),
            Self::U32(value) => _ = write!(shader, "u32({value})"),
            Self::F32(value) => _ = write!(shader, "f32({})", formatting::f32_to_string(*value)),
            Self::Bool(value) => _ = write!(shader, "u32({})", u32::from(*value)),
            Self::Unknown => unreachable!("unknown constants should not be transpiled"),
        }
    }

    pub(crate) fn type_ref(&self) -> Option<&'item StructDefinition> {
        if let Self::TypeRef(type_) = self {
            Some(type_)
        } else {
            None
        }
    }
}
