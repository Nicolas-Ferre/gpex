use crate::compiler::transversal::prelude::PRELUDE_TYPES_FILE_INDEX;
use crate::utils::parsing::span::Span;

const TYPEREF_SIZE: u32 = 8;
const F32_SIZE: u32 = 4;
const I32_SIZE: u32 = 4;
const U32_SIZE: u32 = 4;

#[derive(Debug)]
#[derive_where::derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StructDefinition {
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) scope: Vec<u64>,
    #[derive_where(skip)]
    pub(crate) pub_keyword_span: Option<Span>,
    #[derive_where(skip)]
    pub(crate) name_span: Span,
    #[derive_where(skip)]
    pub(crate) name: String,
    #[derive_where(skip)]
    pub(crate) intrinsic_keyword_span: Span,
}

impl StructDefinition {
    pub(crate) fn dot_path(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn alignment(&self) -> u32 {
        self.size()
    }

    pub(crate) fn size(&self) -> u32 {
        match (self.name_span.file_index, self.name.as_str()) {
            (PRELUDE_TYPES_FILE_INDEX, "typeref") => TYPEREF_SIZE,
            (PRELUDE_TYPES_FILE_INDEX, "f32") => F32_SIZE,
            (PRELUDE_TYPES_FILE_INDEX, "i32") => I32_SIZE,
            (PRELUDE_TYPES_FILE_INDEX, "u32" | "bool") => U32_SIZE,
            _ => unreachable!("not implemented `{}` GPU type", self.name),
        }
    }
}
