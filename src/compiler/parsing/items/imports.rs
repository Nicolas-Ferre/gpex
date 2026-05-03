use crate::compiler::EXT;
use crate::compiler::parsing::patterns::IDENT_PATTERN;
use crate::compiler::parsing::symbols::{
    DOT_SYMBOL, IMPORT_KEYWORD, PUB_KEYWORD, SEMICOLON_SYMBOL, TILDE_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Import {
    pub(crate) id: u64,
    pub(crate) span: Span,
    pub(crate) pub_keyword_span: Option<Span>,
    pub(crate) segments: Vec<ImportSegment>,
    pub(crate) imported_file_index: Option<usize>,
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
            SeparatorParser::None,
            |context| Span::parse_pattern(context, IDENT_PATTERN).map(|_| ()),
        )?;
        let name_segments = context.parse_many(
            |context| Span::parse_pattern(context, IDENT_PATTERN).map(ImportSegment::Name),
            SeparatorParser::NotTrailing(|context| {
                Span::parse_symbol(context, DOT_SYMBOL).map(|_| ())
            }),
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
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ImportSegment {
    Name(Span),
    Parent(Span),
}

impl ImportSegment {
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

    pub(crate) fn span(self) -> Span {
        let (Self::Name(span) | Self::Parent(span)) = self;
        span
    }
}
