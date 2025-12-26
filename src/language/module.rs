use crate::compiler::indexes::Indexes;
use crate::compiletools::parsing::{ParseCtx, ParseError};
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::language::stmts::const_::ConstStmt;
use crate::language::stmts::import::ImportStmt;
use crate::language::stmts::var::VarStmt;

#[derive(Debug)]
pub(crate) struct Module {
    pub(crate) items: Vec<Item>,
}

impl Module {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        let (items, error) = ctx.parse_many(0, usize::MAX, Item::parse, None)?;
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

    pub(crate) fn validate(&self, ctx: &mut ValidateCtx<'_>, indexes: &mut Indexes<'_>) {
        let mut has_invalid_import = false;
        for item in &self.items {
            if let Item::Import(import) = item
                && import.validate(ctx).is_err()
            {
                has_invalid_import = true;
            }
        }
        if has_invalid_import {
            return;
        }
        for item in &self.items {
            _ = item.validate(ctx, indexes);
        }
    }

    pub(crate) fn vars(&self) -> impl Iterator<Item = &VarStmt> {
        self.items.iter().filter_map(|item| {
            if let Item::Var(var) = item {
                Some(var)
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
pub(crate) enum Item {
    Import(ImportStmt),
    Var(VarStmt),
    Const(ConstStmt),
}

impl Item {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        ctx.parse_any(&[
            |ctx| ImportStmt::parse(ctx).map(Self::Import),
            |ctx| VarStmt::parse(ctx).map(Self::Var),
            |ctx| ConstStmt::parse(ctx).map(Self::Const),
        ])
    }

    pub(crate) fn index<'b>(&'b self, indexes: &mut Indexes<'b>) {
        match self {
            Self::Import(item) => item.index(indexes),
            Self::Var(item) => item.index(indexes),
            Self::Const(item) => item.index(indexes),
        }
    }

    pub(crate) fn pre_validate(&self, indexes: &mut Indexes<'_>) {
        match self {
            Self::Import(_) => (),
            Self::Var(item) => item.pre_validate(indexes),
            Self::Const(item) => item.pre_validate(indexes),
        }
    }

    pub(crate) fn validate(
        &self,
        ctx: &mut ValidateCtx<'_>,
        indexes: &mut Indexes<'_>,
    ) -> Result<(), ValidateError> {
        match self {
            Self::Import(_) => Ok(()), // validated during previous pass
            Self::Var(item) => item.validate(ctx, indexes),
            Self::Const(item) => item.validate(ctx, indexes),
        }
    }
}
