use crate::compiler::indexes::{Indexes, Value};
use crate::compiler::transpilation::MAIN_BUFFER_NAME;
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::exprs::Expr;
use crate::language::patterns::IDENT_PAT;
use crate::language::symbols::{EQ_SYM, SEMI_SYM, VAR_SYM};
use crate::validators;
use std::fmt::Write;

#[derive(Debug)]
pub(crate) struct VarStmt {
    pub(crate) id: u64,
    pub(crate) scope: Vec<u64>,
    pub(crate) ident: Span,
    expr: Expr,
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
        indexes.values.register(&self.ident.slice, Value::Var(self));
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        self.expr.pre_validate(indexes);
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::value::check_unique_def(Value::Var(self), &self.ident, ctx, indexes)?;
        validators::value::check_usage(Value::Var(self), &self.ident, ctx, indexes);
        validators::ident::check_letter_count(&self.ident, ctx);
        validators::ident::check_snake_case(&self.ident, ctx);
        self.expr.validate(None, ctx, indexes)?;
        Ok(())
    }

    pub(crate) fn transpile_buffer_field(&self, shader: &mut String) {
        _ = write!(shader, "v{}: i32, ", self.id);
    }

    pub(crate) fn transpile_buffer_init(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.transpile_ref(shader);
        *shader += " = ";
        self.expr.transpile(shader, indexes);
        *shader += "; ";
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String) {
        *shader += MAIN_BUFFER_NAME;
        _ = write!(shader, ".v{}", self.id);
    }

    pub(crate) fn name(&self) -> &str {
        &self.ident.slice
    }
}
