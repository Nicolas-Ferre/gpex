use crate::compiler::indexes::Indexes;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{DOT_SYMBOL, IMPORT_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct Import {
    span: Span,
    segments: Vec<Span>,
    imported_file_index: Option<usize>,
}

impl Import {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let import = Span::parse_symbol(context, IMPORT_KEYWORD)?;
        let segments = Self::parse_segments(context)?;
        let semicolon = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self {
            span: Span {
                file_index: import.file_index,
                start: import.start,
                end: semicolon.end,
            },
            imported_file_index: Self::find_imported_file_index(context, &segments),
            segments,
        })
    }

    fn parse_segments<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Vec<Span>, ParseError<'context>> {
        let (segments, _) = context.parse_many(
            1,
            |context| Span::parse_pattern(context, IDENTIFIER_PATTERN),
            Some(|context| Span::parse_symbol(context, DOT_SYMBOL).map(|_| ())),
        )?;
        Ok(segments)
    }

    fn find_imported_file_index(context: &ParseContext<'_>, segments: &[Span]) -> Option<usize> {
        let dot_path = segments
            .iter()
            .map(|&segment| context.slice(segment))
            .join(".");
        context
            .files
            .iter()
            .position(|file| file.dot_path == dot_path)
    }

    pub(crate) fn index<'index>(&'index self, indexes: &mut Indexes<'index>) {
        if let Some(file_index) = self.imported_file_index {
            indexes.imports.register(self.span.file_index, file_index);
        }
    }

    pub(crate) fn validate(
        &self,
        is_top_import: bool,
        context: &mut ValidateContext<'_>,
    ) -> Result<(), ValidateError> {
        let is_found = self.imported_file_index.is_some();
        validators::import::check_found(is_found, &self.segments, context)?;
        validators::import::check_not_top(is_top_import, self.span, context)?;
        for &segment in &self.segments {
            validators::identifier::check_snake_case(segment, context);
        }
        Ok(())
    }
}
