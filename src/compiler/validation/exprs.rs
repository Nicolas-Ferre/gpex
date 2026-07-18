use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::exprs::literals::{F32Literal, I32Literal, U32Literal};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::validation::{ParamConstness, Validator, validators};
use crate::compiler::values::types::Type;
use crate::utils::validation::ValidateError;

impl Validator<'_, '_> {
    pub(crate) fn validate_expr(&mut self, node: &Expr) -> Result<(), ValidateError> {
        match node {
            Expr::F32Literal(child) => self.validate_f32_literal(child),
            Expr::U32Literal(child) => self.validate_u32_literal(child),
            Expr::I32Literal(child) => self.validate_i32_literal(child),
            Expr::BoolLiteral(_) => Ok(()),
            Expr::Wildcard(span) => {
                validators::expr::report_invalid_wildcard_location(*span, &mut self.context)
            }
            Expr::Call(child) => validators::expr::check_no_return_type(
                child,
                child.span,
                &mut self.context,
                self.indexes,
            )
            .and_then(|()| self.validate_call(child)),
            Expr::Ident(child) => self.validate_ident(child),
        }
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
        let source = self.indexes.sources.get(&node.id).copied();
        let is_constness_ignored = source.is_some_and(ItemRef::is_param_constness_ignored);
        let mut is_error_detected = false;
        for (index, arg) in node.args.iter().enumerate() {
            let param = source.map(|source| &source.params().params[index]);
            let param_const_mark_span = param.and_then(Param::const_mark_span);
            let const_mark_span = if is_constness_ignored {
                None
            } else {
                param_const_mark_span.or(self.const_mark_span)
            };
            let param_constness = if param_const_mark_span.is_some() {
                ParamConstness::ExplicitOnly
            } else {
                self.param_constness
            };
            self.run_with_param_constness(param_constness, |self_| {
                self_.with_const_mark_span(const_mark_span, |self_| {
                    is_error_detected |= self_.validate_expr(&arg.value).is_err(); // no-fn-check (recursivity)
                });
            });
        }
        if is_error_detected {
            return Err(ValidateError);
        }
        let source = validators::item::check_found(
            source,
            node,
            node.span,
            &node.key(),
            &self.key_renderer.call_key(node)?,
            &mut self.context,
            self.indexes,
        )?;
        for (arg, param) in node.args.iter().zip(&source.params().params) {
            // Error is ignored because it is isolated from other errors
            _ = validators::expr::check_arg_name(arg, param, &mut self.context);
        }
        if let Some(const_mark_span) = self.const_mark_span {
            validators::expr::check_const_value(
                source,
                node.span,
                const_mark_span,
                &mut self.context,
                self.param_constness,
            )?;
        }
        validators::expr::check_f32_const_bounds(
            self.value_resolver.is_const_infinite_f32(node),
            node.span,
            &mut self.context,
        )?;
        self.validate_mul_add_candidate(node, source);
        Ok(())
    }

    pub(crate) fn validate_ident(&mut self, node: &Ident) -> Result<(), ValidateError> {
        let source = self.indexes.sources.get(&node.id).copied();
        let source = validators::item::check_found(
            source,
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
                self.param_constness,
            )?;
        }
        Ok(())
    }

    fn validate_mul_add_candidate(&mut self, node: &Call, source: ItemRef<'_>) {
        let f32_type = self.indexes.search_prelude_type("f32");
        let are_all_args_f32 = node
            .args
            .iter()
            .all(|arg| self.value_resolver.expr_type(&arg.value) == Type::Struct(f32_type));
        validators::expr::check_mul_add_candidate(
            source,
            node,
            are_all_args_f32,
            &mut self.context,
            self.indexes,
        );
    }
}
