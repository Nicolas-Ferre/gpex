use crate::compiler::constants::ConstValue;
use crate::compiler::indexes::{Indexes, Value};
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::exprs::Expr;
use crate::language::patterns::IDENT_PAT;
use crate::language::symbols::{CONST_SYM, EQ_SYM, SEMI_SYM};
use crate::validators;

#[derive(Debug)]
pub(crate) struct ConstStmt {
    pub(crate) id: u64,
    pub(crate) scope: Vec<u64>,
    pub(crate) const_: Span,
    pub(crate) ident: Span,
    expr: Expr,
}

impl<'a> ConstStmt {
    pub(crate) fn parse(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        ctx.define_scope(|ctx, id| {
            let const_ = Span::parse_symbol(ctx, CONST_SYM)?;
            let ident = Span::parse_pattern(ctx, IDENT_PAT)?;
            let _ = Span::parse_symbol(ctx, EQ_SYM)?;
            let expr = Expr::parse(ctx)?;
            let _ = Span::parse_symbol(ctx, SEMI_SYM)?;
            Ok(Self {
                id,
                scope: ctx.scope().to_vec(),
                const_,
                ident,
                expr,
            })
        })
    }

    pub(crate) fn index<'b>(&'b self, indexes: &mut Indexes<'b>) {
        indexes
            .values
            .register(&self.ident.slice, Value::Const(self));
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        self.expr.pre_validate(indexes);
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::value::check_unique_def(Value::Const(self), &self.ident, ctx, indexes)?;
        validators::value::check_usage(Value::Const(self), &self.ident, ctx, indexes);
        validators::ident::check_letter_count(&self.ident, ctx);
        validators::ident::check_screaming_snake_case(&self.ident, ctx);
        self.expr.validate(Some(&self.const_), ctx, indexes)?;
        Ok(())
    }

    #[expect(clippy::expect_used)] // validated before
    pub(crate) fn const_value(&self, indexes: &Indexes<'_>) -> ConstValue {
        self.expr
            .const_value(indexes)
            .expect("internal error: invalid constant value")
    }

    pub(crate) fn transpile_ref(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.const_value(indexes).transpile(shader);
    }
}
