use crate::compiler::indexes::Indexes;
use crate::compiletools::indexing::Node;
use crate::compiletools::logs::{Log, LogLevel};
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::patterns::IDENT_PAT;

#[derive(Debug)]
pub(crate) struct IdentExpr {
    id: u64,
    scope: Vec<u64>,
    ident: Span,
}

impl Node for IdentExpr {
    fn file_index(&self) -> usize {
        self.ident.file_index
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn scope(&self) -> &[u64] {
        &self.scope
    }
}

impl IdentExpr {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        Ok(Self {
            id: ctx.next_id(),
            scope: ctx.scope().to_vec(),
            ident: Span::parse_pattern(ctx, IDENT_PAT)?,
        })
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        if let Some(source) = indexes.values.search(&self.ident.slice, self) {
            indexes.value_sources.insert(self.id, source);
            indexes
                .item_first_ref
                .insert(source.id(), self.ident.clone());
        }
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        if indexes.value_sources.contains_key(&self.id) {
            Ok(())
        } else {
            ctx.logs.push(Log {
                level: LogLevel::Error,
                msg: format!("`{}` value not found", self.ident.slice),
                loc: Some(ctx.loc(&self.ident)),
                inner: vec![],
            });
            Err(ValidateError)
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        indexes.value_sources[&self.id].transpile_call(shader);
    }
}
