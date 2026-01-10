use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::expressions::Expression;
use crate::language::symbols::{RETURN_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};

#[derive(Debug)]
pub(crate) struct ReturnStatement {
    pub(crate) span: Span,
    pub(crate) value: Expression,
}

impl ReturnStatement {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let return_keyword_span = Span::parse_symbol(context, RETURN_KEYWORD)?;
        let value = Expression::parse(context)?;
        let semicolon_keyword_span = Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self {
            span: return_keyword_span.until(semicolon_keyword_span),
            value,
        })
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        self.value.index(indexes);
    }

    pub(crate) fn validate(
        &self,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.value.validate(None, context, indexes)?;
        Ok(())
    }

    pub(crate) fn dependencies<'index>(
        &self,
        dependencies: Dependencies<'index>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<'index>, Vec<Span>> {
        self.value.dependencies(dependencies, indexes)
    }
}
