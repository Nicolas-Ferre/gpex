use crate::compiletools::logs::{Log, LogLevel};
use crate::compiletools::parsing::{ParseCtx, ParseError, Span};
use crate::compiletools::validation::ValidateCtx;
use crate::language::patterns::I32_LIT_PAT;

#[derive(Debug)]
pub(crate) struct I32Lit {
    span: Span,
}

impl I32Lit {
    pub(crate) fn parse<'a>(ctx: &mut ParseCtx<'a>) -> Result<Self, ParseError<'a>> {
        Ok(Self {
            span: Span::parse_pattern(ctx, I32_LIT_PAT)?,
        })
    }

    pub(crate) fn validate(&self, ctx: &mut ValidateCtx<'_>) {
        if self.cleaned_slice().parse::<i32>().is_err() {
            ctx.logs.push(Log {
                level: LogLevel::Error,
                msg: "`i32` literal out of bounds".into(),
                loc: Some(ctx.loc(&self.span)),
                inner: vec![],
            });
        }
    }

    pub(crate) fn transpile(&self, shader: &mut String) {
        *shader += "i32(";
        *shader += &self.cleaned_slice();
        *shader += ")";
    }

    fn cleaned_slice(&self) -> String {
        self.span.slice.replace('_', "")
    }
}
