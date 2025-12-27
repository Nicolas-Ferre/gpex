use crate::compiler::constants::ConstValue;
use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::{Indexes, Value};
use crate::compiletools::indexing::NodeRef;
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::patterns::IDENT_PAT;
use crate::validators;

#[derive(Debug)]
pub(crate) struct IdentExpr {
    id: u64,
    scope: Vec<u64>,
    span: Span,
}

impl NodeRef for &IdentExpr {
    fn file_index(&self) -> usize {
        self.span.file_index
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
            span: Span::parse_pattern(ctx, IDENT_PAT)?,
        })
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        if let Some(source) = indexes
            .values
            .search(&self.span.slice, self, &indexes.imports)
        {
            indexes.value_sources.insert(self.id, source);
            indexes
                .item_first_ref
                .entry(source.id())
                .or_insert_with(|| self.span.clone());
        }
    }

    pub(crate) fn dependencies<'a>(
        &self,
        dependencies: Dependencies<'a>,
        indexes: &Indexes<'a>,
    ) -> Result<Dependencies<'a>, Vec<Span>> {
        if let Some(&source) = indexes.value_sources.get(&self.id) {
            let dependencies = dependencies.register(self.span.clone(), source)?;
            source.dependencies(dependencies, indexes)
        } else {
            Ok(dependencies)
        }
    }

    pub(crate) fn validate(
        &self,
        const_span: Option<&Span>,
        ctx: &mut ValidateCtx<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::value::check_found(self, &self.span, ctx, indexes)?;
        if let Some(const_span) = const_span {
            validators::ident::check_const(self, &self.span, const_span, ctx, indexes)?;
        }
        Ok(())
    }

    fn source<'a>(&self, indexes: &Indexes<'a>) -> Value<'a> {
        indexes.value_sources[&self.id]
    }

    pub(crate) fn const_value(&self, indexes: &Indexes<'_>) -> Option<ConstValue> {
        self.source(indexes).const_value(indexes)
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.source(indexes).transpile_ref(shader, indexes);
    }
}
