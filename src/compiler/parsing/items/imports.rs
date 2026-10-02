use crate::compiler::transversal::ast::items::imports::{Import, ImportSegment};
use crate::compiler::transversal::ast::patterns::IDENT_PATTERN;
use crate::compiler::transversal::ast::symbols::{
    DOT_SYMBOL, IMPORT_KEYWORD, PUB_KEYWORD, SEMICOLON_SYMBOL, TILDE_SYMBOL,
};
use crate::utils::parsing::context::{ParseContext, SeparatorParser};
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;

pub(crate) fn parse<'context>(
    context: &mut ParseContext<'context>,
) -> Result<Import, ParseError<'context>> {
    let pub_keyword_span = Span::parse_symbol(context, PUB_KEYWORD).ok();
    let import = Span::parse_symbol(context, IMPORT_KEYWORD)?;
    context.force_parse_any_error();
    let segments = parse_segments(context)?;
    let semicolon = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
    Ok(Import {
        id: context.next_id(),
        span: Span {
            file_index: import.file_index,
            start: import.start,
            end: semicolon.end,
        },
        pub_keyword_span,
        imported_file_index: find_imported_file_index(context, &segments),
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
        SeparatorParser::NotTrailing(|context| Span::parse_symbol(context, DOT_SYMBOL).map(|_| ())),
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
