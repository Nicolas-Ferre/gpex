use crate::compiler::indexes::Indexes;
use crate::language::DependencyType;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::symbols::{EQUAL_SYMBOL, SEMICOLON_SYMBOL};
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::{ParseContext, ParseError, Span};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

#[derive(Debug)]
pub(crate) struct AssignmentStatement {
    pub(crate) assigned: Expression,
    pub(crate) value: Expression,
}

impl AssignmentStatement {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let assigned = Expression::parse(context)?;
        Span::parse_symbol(context, EQUAL_SYMBOL)?;
        context.force_parse_any_error();
        let value = Expression::parse(context)?;
        Span::parse_symbol(context, SEMICOLON_SYMBOL)?;
        Ok(Self { assigned, value })
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        self.assigned.index(indexes);
        self.value.index(indexes);
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        let dependencies = self.assigned.dependencies(type_, dependencies, indexes)?;
        self.value.dependencies(type_, dependencies, indexes)
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        let assigned_result = self.validate_assigned(constant_mark_span, context, indexes);
        let value_result = self.validate_value(constant_mark_span, context, indexes);
        assigned_result.and(value_result)
    }

    fn validate_assigned(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        validators::expression::check_ref(&self.assigned, context, indexes);
        self.assigned
            .validate(constant_mark_span, context, indexes)?;
        Ok(())
    }

    fn validate_value(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        self.value.validate(constant_mark_span, context, indexes)?;
        validators::expression::check_types(
            self.value.span(),
            self.value.type_(indexes),
            Some(self.assigned.span()),
            self.assigned.type_(indexes),
            context,
        )?;
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        self.assigned.transpile(shader, indexes);
        *shader += " = ";
        self.value.transpile(shader, indexes);
        *shader += ";";
    }
}
