use crate::compiler::EXTENSION;
use crate::compiler::indexes::Indexes;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{DOT_SYMBOL, IMPORT_KEYWORD, SEMICOLON_SYMBOL, TILDE_SYMBOL};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Import {
    span: Span,
    segments: Vec<ImportSegment>,
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
    ) -> Result<Vec<ImportSegment>, ParseError<'context>> {
        #[expect(clippy::expect_used)] // as this part is optional, parsing shouldn't fail
        let (mut segments, _) = context
            .parse_many(
                0,
                |context| Span::parse_symbol(context, TILDE_SYMBOL).map(ImportSegment::Parent),
                Some(|context| Span::parse_symbol(context, DOT_SYMBOL).map(|_| ())),
            )
            .expect("internal error: import tilde parsing failed");
        if !segments.is_empty() {
            Span::parse_symbol(context, DOT_SYMBOL)?;
        }
        let (name_segments, _) = context.parse_many(
            1,
            |context| Span::parse_pattern(context, IDENTIFIER_PATTERN).map(ImportSegment::Name),
            Some(|context| Span::parse_symbol(context, DOT_SYMBOL).map(|_| ())),
        )?;
        segments.extend(name_segments);
        Ok(segments)
    }

    fn find_imported_file_index(
        context: &ParseContext<'_>,
        segments: &[ImportSegment],
    ) -> Option<usize> {
        let fs_path = ImportSegment::fs_path(segments, context, context.root_path);
        context
            .files
            .iter()
            .position(|file| file.fs_path == fs_path)
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
        validators::import::check_top(is_top_import, self.span, context)?;
        validators::import::check_self_import(self.imported_file_index, self.span, context);
        for &segment in &self.segments {
            if let ImportSegment::Name(span) = segment {
                validators::identifier::check_snake_case(span, context);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ImportSegment {
    Name(Span),
    Parent(Span),
}

impl ImportSegment {
    pub(crate) fn span(self) -> Span {
        let (Self::Name(span) | Self::Parent(span)) = self;
        span
    }

    pub(crate) fn fs_path(
        segments: &[Self],
        span_properties: &impl SpanProperties,
        root_path: &Path,
    ) -> PathBuf {
        let mut parent_segment_count = 0;
        let mut path = match segments[0] {
            Self::Name(_) => root_path.to_path_buf(),
            Self::Parent(_) => span_properties.fs_path(segments[0].span()).to_path_buf(),
        };
        for &segment in segments {
            match segment {
                Self::Name(span) => path.push(span_properties.slice(span)),
                Self::Parent(_) => {
                    if parent_segment_count < path.iter().count()
                        && let Some(parent) = path.parent()
                    {
                        path = parent.to_path_buf();
                    } else {
                        path.push("..");
                        parent_segment_count += 1;
                    }
                }
            }
        }
        path.with_extension(EXTENSION)
    }
}
