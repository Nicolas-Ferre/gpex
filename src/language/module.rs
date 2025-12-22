use crate::compiler::indexes::Indexes;
use crate::compiletools::parsing::{ParseCtx, ParseError};
use crate::compiletools::validation::ValidateCtx;
use crate::language::var_stmt::VarStmt;

#[derive(Debug)]
pub(crate) struct Module {
    pub(crate) items: Vec<VarStmt>,
}

impl Module {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        let (items, error) = ctx.parse_many(0, usize::MAX, VarStmt::parse)?;
        if let Some(error) = error {
            return Err(error);
        }
        Ok(Self { items })
    }

    pub(crate) fn index<'b>(&'b self, indexes: &mut Indexes<'b>) {
        for item in &self.items {
            item.index(indexes);
        }
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        for item in &self.items {
            item.pre_validate(indexes);
        }
    }

    pub(crate) fn validate(&self, ctx: &mut ValidateCtx<'_>, indexes: &Indexes<'_>) {
        for item in &self.items {
            item.validate(ctx, indexes);
        }
    }

    pub(crate) fn transpile_buffer_fields(&self, shader: &mut String) {
        for item in &self.items {
            item.transpile_buffer_field(shader);
        }
    }

    pub(crate) fn transpile_buffer_init(&self, shader: &mut String, indexes: &Indexes<'_>) {
        for item in &self.items {
            item.transpile_buffer_init(shader, indexes);
        }
    }
}
