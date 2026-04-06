use crate::compiler::EXT;
use crate::compiler::indexes::Indexes;
use crate::language::patterns::IDENT_PATTERN;
use crate::language::symbols::{
    DOT_SYMBOL, IMPORT_KEYWORD, PUB_KEYWORD, SEMICOLON_SYMBOL, TILDE_SYMBOL,
};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use crate::validators::ident::Case;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Import {
    id: u64,
    span: Span,
    pub_keyword_span: Option<Span>,
    segments: Vec<ImportSegment>,
    imported_file_index: Option<usize>,
}

impl Import {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
        let import = Span::parse_symbol(context, IMPORT_KEYWORD)?;
        context.force_parse_any_error();
        let segments = Self::parse_segments(context)?;
        let semicolon = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self {
            id: context.next_id(),
            span: Span {
                file_index: import.file_index,
                start: import.start,
                end: semicolon.end,
            },
            pub_keyword_span,
            imported_file_index: Self::find_imported_file_index(context, &segments),
            segments,
        })
    }

    fn parse_segments<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Vec<ImportSegment>, ParseError<'context>> {
        let mut segments = context.parse_many(
            |context| {
                let tilde = Span::parse_symbol(context, TILDE_SYMBOL)?;
                Span::parse_symbol(context, DOT_SYMBOL)?;
                Ok(ImportSegment::Parent(tilde))
            },
            None,
            |context| Span::parse_pattern(context, IDENT_PATTERN).map(|_| ()),
        )?;
        let name_segments = context.parse_many(
            |context| Span::parse_pattern(context, IDENT_PATTERN).map(ImportSegment::Name),
            Some(|context| Span::parse_symbol(context, DOT_SYMBOL).map(|_| ())),
            |context| Span::parse_symbol(context, SEMICOLON_SYMBOL).map(|_| ()),
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
            let is_pub = self.pub_keyword_span.is_some();
            indexes.imports.register(
                Some(self.id),
                Some(self.span),
                self.span.file_index,
                file_index,
                is_pub,
            );
        }
    }

    pub(crate) fn validate(
        &self,
        is_top_import: bool,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let is_found = self.imported_file_index.is_some();
        let is_pub = self.pub_keyword_span.is_some();
        validators::import::check_found(is_found, &self.segments, context)?;
        validators::import::check_top(is_top_import, self.span, context)?;
        validators::import::check_self_import(self.imported_file_index, self.span, context);
        validators::import::check_usage(
            self.id,
            self.imported_file_index,
            self.span,
            is_pub,
            &self.segments,
            context,
            indexes,
        );
        for &segment in &self.segments {
            if let ImportSegment::Name(span) = segment {
                validators::ident::check_case(span, &[Case::Snake], context);
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
        span_props: &impl SpanProps,
        root_path: &Path,
    ) -> PathBuf {
        let mut parent_segment_count = 0;
        let mut path = match segments[0] {
            Self::Name(_) => root_path.to_path_buf(),
            Self::Parent(_) => span_props.fs_path(segments[0].span()).to_path_buf(),
        };
        for &segment in segments {
            match segment {
                Self::Name(span) => path.push(span_props.slice(span)),
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
        path.with_extension(EXT)
    }
}
