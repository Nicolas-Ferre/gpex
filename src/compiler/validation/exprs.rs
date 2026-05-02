use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::validation::{Validator, validators};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

impl Validator<'_, '_> {
    pub(crate) fn validate_expr(
        &mut self,
        node: &Expr,
        const_mark_span: Option<Span>,
    ) -> Result<(), ValidateError> {
        let previous_const_mark_span = self.const_mark_span;
        self.const_mark_span = const_mark_span;
        let result = match node {
            Expr::F32Literal(child) => self.validate_f32_literal(child),
            Expr::U32Literal(child) => self.validate_u32_literal(child),
            Expr::I32Literal(child) => self.validate_i32_literal(child),
            Expr::BoolLiteral(_) => Ok(()),
            Expr::Call(child) => validators::expr::check_no_return_type(
                child,
                child.span,
                &mut self.context,
                self.indexes,
            )
            .and_then(|()| self.validate_call(child)),
            Expr::Ident(child) => self.validate_ident(child),
        };
        self.const_mark_span = previous_const_mark_span;
        result
    }

    pub(crate) fn validate_f32_literal(&mut self, node: &F32Literal) -> Result<(), ValidateError> {
        validators::literal::check_bounds(node.value.is_some(), node.span, "f32", &mut self.context)
    }

    pub(crate) fn validate_i32_literal(&mut self, node: &I32Literal) -> Result<(), ValidateError> {
        validators::literal::check_bounds(node.value.is_some(), node.span, "i32", &mut self.context)
    }

    pub(crate) fn validate_u32_literal(&mut self, node: &U32Literal) -> Result<(), ValidateError> {
        validators::literal::check_bounds(node.value.is_some(), node.span, "u32", &mut self.context)
    }

    pub(crate) fn validate_call(&mut self, node: &Call) -> Result<(), ValidateError> {
        for arg in &node.args {
            self.validate_expr(arg, self.const_mark_span)?; // no-fn-check (recursivity)
        }
        let Some(fn_key) = self.key_renderer.call_key(node) else {
            return Err(ValidateError);
        };
        let source = validators::item::check_found(
            node,
            node.span,
            &node.key(),
            &fn_key,
            &mut self.context,
            self.indexes,
        )?;
        if let Some(const_mark_span) = self.const_mark_span {
            validators::expr::check_const_value(
                source,
                node.span,
                const_mark_span,
                &mut self.context,
                &self.const_checker,
            )?;
        }
        Ok(())
    }

    pub(crate) fn validate_ident(&mut self, node: &Ident) -> Result<(), ValidateError> {
        let source = validators::item::check_found(
            node,
            node.span,
            &node.slice,
            &node.slice,
            &mut self.context,
            self.indexes,
        )?;
        if let Some(const_mark_span) = self.const_mark_span {
            validators::expr::check_const_value(
                source,
                node.span,
                const_mark_span,
                &mut self.context,
                &self.const_checker,
            )?;
        }
        Ok(())
    }
}
