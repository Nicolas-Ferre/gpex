use crate::compiler::consts::ConstContext;
use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::language::symbols::{RETURN_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use crate::utils::validation::{ValidateContext, ValidateError};

#[derive(Debug)]
pub(crate) struct ReturnStatement {
    pub(crate) span: Span,
    pub(crate) value: Expr,
}

impl ReturnStatement {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let return_keyword_span = Span::parse_symbol(context, RETURN_KEYWORD)?;
        context.force_parse_any_error();
        let value = Expr::parse(context)?;
        let semicolon_keyword_span = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self {
            span: return_keyword_span.until(semicolon_keyword_span),
            value,
        })
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        self.value.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        self.value.dependencies(type_, dependencies, indexes)
    }

    pub(crate) fn validate(
        &self,
        const_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.value.validate(const_mark_span, context, indexes)?;
        Ok(())
    }

    pub(crate) fn transpile<'index>(
        &self,
        shader: &mut String,
        indexes: &Indexes<'index>,
        context: &mut ConstContext<'index>,
    ) {
        *shader += "return ";
        self.value.transpile(shader, indexes, context);
        *shader += ";";
    }
}
