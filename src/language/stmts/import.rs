use crate::compiler::indexes::Indexes;
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::patterns::IDENT_PAT;
use crate::language::symbols::{DOT_SYM, IMPORT_SYM, SEMI_SYM};
use crate::validators;
use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct ImportStmt {
    span: Span,
    segments: Vec<Span>,
    imported_file_index: Option<usize>,
}

impl ImportStmt {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        let import = Span::parse_symbol(ctx, IMPORT_SYM)?;
        let (segments, _) = ctx.parse_many(
            1,
            usize::MAX,
            |ctx| Span::parse_pattern(ctx, IDENT_PAT),
            Some(|ctx| Span::parse_symbol(ctx, DOT_SYM).map(|_| ())),
        )?;
        let semi = Span::parse_symbol(ctx, SEMI_SYM)?;
        let span = Span {
            file_index: import.file_index,
            start: import.start,
            end: semi.end,
            slice: String::new(),
        };
        let dot_path = segments.iter().map(|segment| &segment.slice).join(".");
        let imported_file_index = ctx.files.iter().position(|file| file.dot_path == dot_path);
        Ok(Self {
            span,
            segments,
            imported_file_index,
        })
    }

    pub(crate) fn index<'b>(&'b self, indexes: &mut Indexes<'b>) {
        if let Some(imported_file_index) = self.imported_file_index {
            indexes
                .imports
                .register(self.span.file_index, imported_file_index);
        }
    }

    pub(crate) fn validate(
        &self,
        is_top_import: bool,
        ctx: &mut ValidateCtx<'_>,
    ) -> Result<(), ValidateError> {
        validators::import::check_found(self.imported_file_index.is_some(), &self.segments, ctx)?;
        validators::import::check_not_top(is_top_import, &self.span, ctx)?;
        for segment in &self.segments {
            validators::ident::check_snake_case(segment, ctx);
        }
        Ok(())
    }
}
