use crate::compiler::indexes::Indexes;
use crate::compiler::transpilation::MAIN_BUFFER_NAME;
use crate::compiletools::indexing::Node;
use crate::compiletools::logs::{Log, LogInner, LogLevel};
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::ValidateCtx;
use crate::language::exprs::Expr;
use crate::language::patterns::IDENT_PAT;
use crate::language::symbols::{EQ_SYM, SEMI_SYM, VAR_SYM};
use std::fmt::Write;

#[derive(Debug)]
pub(crate) struct VarStmt {
    id: u64,
    scope: Vec<u64>,
    ident: Span,
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
                scope: ctx.scope().clone(),
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

    pub(crate) fn validate(&self, ctx: &mut ValidateCtx<'_>, indexes: &mut Indexes<'_>) {
        let var_name = &self.ident.slice;
        // TODO: add validator module at root of the crate to encourage reuse of errors
        if let Some(duplicated_item) = indexes.values.search(var_name, self) {
            ctx.logs.push(Log {
                level: LogLevel::Error,
                msg: format!("`{var_name}` item defined multiple times"),
                loc: Some(ctx.loc(&self.ident)),
                inner: vec![LogInner {
                    level: LogLevel::Info,
                    msg: "item also defined here".into(),
                    loc: Some(ctx.loc(&duplicated_item.ident)),
                }],
            });
        } else {
            let ref_span = indexes.item_first_ref.get(&self.id);
            if ref_span.is_none() && !var_name.starts_with('_') {
                ctx.logs.push(Log {
                    level: LogLevel::Warning,
                    msg: format!("`{var_name}` variable unused"),
                    loc: Some(ctx.loc(&self.ident)),
                    inner: vec![],
                });
            } else if let Some(ref_span) = ref_span
                && var_name.starts_with('_')
            {
                ctx.logs.push(Log {
                    level: LogLevel::Warning,
                    msg: format!("`{var_name}` variable used but name starting with `_`"),
                    loc: Some(ctx.loc(&self.ident)),
                    inner: vec![LogInner {
                        level: LogLevel::Info,
                        msg: "variable used here".into(),
                        loc: Some(ctx.loc(ref_span)),
                    }],
                });
            }
            if var_name.len() == 1 && var_name != "_" {
                ctx.logs.push(Log {
                    level: LogLevel::Warning,
                    msg: format!("`{var_name}` variable name is single letter"),
                    loc: Some(ctx.loc(&self.ident)),
                    inner: vec![],
                });
            } else if !is_snake_case(var_name) {
                ctx.logs.push(Log {
                    level: LogLevel::Warning,
                    msg: format!("`{var_name}` variable name not in snake_case"),
                    loc: Some(ctx.loc(&self.ident)),
                    inner: vec![],
                });
            }
        }
        let _ = self.expr.validate(ctx, indexes);
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

fn is_snake_case(ident: &str) -> bool {
    ident
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_numeric() || c == '_')
}
