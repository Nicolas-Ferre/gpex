use crate::compiler::indexes::Indexes;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{
    CLOSE_BRACE_SYMBOL, COMPILERIMPL_KEYWORD, EQUAL_SYMBOL, OPEN_BRACE_SYMBOL, PUB_KEYWORD,
    STRUCT_KEYWORD,
};
use crate::utils::endianness;
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use std::fmt::Write;

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
    name: String,
}

impl StructDefinition {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        context.define_scope(|context, id| {
            let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
            Span::parse_symbol(context, STRUCT_KEYWORD)?;
            let name_span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
            Span::parse_symbol(context, EQUAL_SYMBOL)?;
            Span::parse_symbol(context, COMPILERIMPL_KEYWORD)?;
            Span::parse_symbol(context, OPEN_BRACE_SYMBOL)?;
            Span::parse_symbol(context, CLOSE_BRACE_SYMBOL)?;
            Ok(Self {
                id,
                scope: context.scope().to_vec(),
                pub_keyword_span,
                name_span,
                name: context.slice(name_span).into(),
            })
        })
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(&self.name, ItemRef::Struct(self));
    }

    pub(crate) fn index_ref<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.types.insert(self);
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::item::check_prelude_location(ItemRef::Struct(self), context)?;
        Ok(())
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index Self {
        indexes.search_prelude_type("typeref")
    }

    pub(crate) fn dot_path(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn size(&self) -> u32 {
        match (self.name_span.file_index, self.name.as_str()) {
            (PRELUDE_FILE_INDEX, "typeref") => TYPEREF_SIZE,
            (PRELUDE_FILE_INDEX, "f32") => F32_SIZE,
            (PRELUDE_FILE_INDEX, "i32") => I32_SIZE,
            (PRELUDE_FILE_INDEX, "u32" | "bool") => U32_SIZE,
            _ => unreachable!("not implemented GPU type"),
        }
    }

    pub(crate) fn alignment(&self) -> u32 {
        self.size()
    }

    pub(crate) fn transpile_name(&self) -> String {
        match (self.name_span.file_index, self.name.as_str()) {
            (PRELUDE_FILE_INDEX, "typeref") => "vec2<u32>".into(),
            (PRELUDE_FILE_INDEX, "f32") => "f32".into(),
            (PRELUDE_FILE_INDEX, "i32") => "i32".into(),
            (PRELUDE_FILE_INDEX, "u32" | "bool") => "u32".into(),
            _ => unreachable!("not implemented GPU type"),
        }
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String) {
        let [id_part1, id_part2] = endianness::to_portable_u32x2(self.id);
        _ = write!(shader, "vec2<u32>({id_part1}, {id_part2})");
    }
}
