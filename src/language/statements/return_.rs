use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::symbols::{RETURN_KEYWORD, SEMICOLON_SYMBOL};
use crate::utils::dependencies::Dependencies;
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
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.value.validate(constant_mark_span, context, indexes)?;
        Ok(())
    }

    pub(crate) fn is_global_variable_modified(&self, indexes: &Indexes<'_>) -> bool {
        self.value.is_global_variable_modified(indexes)
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        *shader += "return ";
        self.value.transpile(shader, indexes);
        *shader += ";";
    }
}
