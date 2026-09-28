use crate::compiler::transversal::ast::items::types::StructDefinition;
use crate::compiler::transversal::prelude::files::PRELUDE_TYPES_FILE_INDEX;

const TYPEREF_SIZE: u32 = 8;
const F32_SIZE: u32 = 4;
const I32_SIZE: u32 = 4;
const U32_SIZE: u32 = 4;

pub(crate) fn alignment(struct_: &StructDefinition) -> u32 {
    size(struct_)
}

pub(crate) fn size(struct_: &StructDefinition) -> u32 {
    match (struct_.name_span.file_index, struct_.name.as_str()) {
        (PRELUDE_TYPES_FILE_INDEX, "typeref") => TYPEREF_SIZE,
        (PRELUDE_TYPES_FILE_INDEX, "f32") => F32_SIZE,
        (PRELUDE_TYPES_FILE_INDEX, "i32") => I32_SIZE,
        (PRELUDE_TYPES_FILE_INDEX, "u32" | "bool") => U32_SIZE,
        _ => unreachable!("not implemented `{}` GPU type", struct_.name),
    }
}
