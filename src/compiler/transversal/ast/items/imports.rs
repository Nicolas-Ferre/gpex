use crate::compiler::EXT;
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
                Self::Parent(_) => Self::push_parent(&mut path, &mut parent_segment_count),
            }
        }
        path.with_extension(EXT)
    }

    pub(crate) fn span(self) -> Span {
        let (Self::Name(span) | Self::Parent(span)) = self;
        span
    }

    fn push_parent(path: &mut PathBuf, parent_segment_count: &mut usize) {
        if *parent_segment_count < path.iter().count()
            && let Some(parent) = path.parent()
        {
            *path = parent.to_path_buf();
        } else {
            path.push("..");
            *parent_segment_count += 1;
        }
    }
}
