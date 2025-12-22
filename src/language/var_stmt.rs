use crate::compiler::indexes::Indexes;
use crate::compiler::transpilation::MAIN_BUFFER_NAME;
use crate::compiletools::indexing::Node;
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::exprs::Expr;
use crate::language::patterns::IDENT_PAT;
use crate::language::symbols::{EQ_SYM, SEMI_SYM, VAR_SYM};
use crate::validators;
use std::fmt::Write;

#[derive(Debug)]
pub(crate) struct VarStmt {
    id: u64,
    scope: Vec<u64>,
    pub(crate) ident: Span,
    expr: Expr,
}

impl Node for VarStmt {
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

impl<'a> VarStmt {
    pub(crate) fn parse(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        ctx.define_scope(|ctx, id| {
            let _ = Span::parse_symbol(ctx, VAR_SYM)?;
            let ident = Span::parse_pattern(ctx, IDENT_PAT)?;
            let _ = Span::parse_symbol(ctx, EQ_SYM)?;
            let expr = Expr::parse(ctx)?;
            let _ = Span::parse_symbol(ctx, SEMI_SYM)?;
            Ok(Self {
                id,
                scope: ctx.scope().to_vec(),
                ident,
                expr,
            })
        })
    }

    pub(crate) fn index<'b>(&'b self, indexes: &mut Indexes<'b>) {
        indexes.values.register(&self.ident.slice, self);
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        self.expr.pre_validate(indexes);
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::value::check_unique_def(self, &self.ident, ctx, indexes)?;
        validators::value::check_usage(self, &self.ident, ctx, indexes);
        validators::ident::check_letter_count(&self.ident, ctx);
        validators::ident::check_snake_case(&self.ident, ctx);
        self.expr.validate(ctx, indexes)?;
        Ok(())
    }

    pub(crate) fn transpile_buffer_field(&self, shader: &mut String) {
        _ = write!(shader, "v{}: i32, ", self.id());
    }

    pub(crate) fn transpile_buffer_init(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.transpile_call(shader);
        *shader += " = ";
        self.expr.transpile(shader, indexes);
        *shader += "; ";
    }

    pub(crate) fn transpile_call(&self, shader: &mut String) {
        *shader += MAIN_BUFFER_NAME;
        _ = write!(shader, ".v{}", self.id());
    }

    pub(crate) fn name(&self) -> &str {
        &self.ident.slice
    }
}
