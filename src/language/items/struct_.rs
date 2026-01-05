use crate::compiler::indexes::Indexes;
use crate::compiler::prelude::{PRELUDE_FILE_INDEX, PreludeEndLocation};
use crate::language::items::ItemRef;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{
    CLOSE_BRACE_SYMBOL, COMPILERIMPL_KEYWORD, EQUAL_SYMBOL, OPEN_BRACE_SYMBOL, PUB_KEYWORD,
    STRUCT_KEYWORD,
};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use std::fmt::Write;

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
    type_index: u32,
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
                type_index: context.next_type_index(),
            })
        })
    }

    pub(crate) fn index_item<'index>(&'index self, indexes: &mut Indexes<'index>) {
        indexes.items.register(&self.name, ItemRef::Struct(self));
        debug_assert_eq!(indexes.types.len(), self.type_index as usize);
        indexes.types.push(self);
    }

    pub(crate) fn validate(&self, context: &mut ValidateContext<'_>) -> Result<(), ValidateError> {
        validators::item::check_prelude_location(ItemRef::Struct(self), context)?;
        Ok(())
    }

    pub(crate) fn type_<'index>(indexes: &Indexes<'index>) -> &'index Self {
        match indexes
            .items
            .search("typeref", PreludeEndLocation, &indexes.imports, false)
        {
            Some(ItemRef::Struct(item)) => item,
            Some(_) | None => unreachable!("missing `typeref` type in prelude"),
        }
    }

    pub(crate) fn dot_path(&self) -> String {
        self.name.clone()
    }

    #[expect(clippy::unused_self)] // will be used in the future
    pub(crate) fn size(&self) -> u32 {
        4
    }

    pub(crate) fn transpile_name(&self) -> String {
        if self.name_span.file_index == PRELUDE_FILE_INDEX && self.name == "typeref" {
            "u32".into()
        } else {
            "i32".into()
        }
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String) {
        _ = write!(shader, "u32({})", self.type_index);
    }
}
