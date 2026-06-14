use crate::compiler::dependencies::DependencyResolver;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::statements::{AssignmentStatement, Statement};
use crate::compiler::validation::{ParamConstness, Validator, validators};
use crate::compiler::values::types::Type;
use crate::utils::validation::ValidateError;

impl<'item> Validator<'item, '_> {
    pub(super) fn validate_fn(&mut self, node: &'item FnDefinition) -> Result<(), ValidateError> {
        let ref_ = ItemRef::Fn(node);
        let compilerimpl_span = node.body.compilerimpl_keyword_span();
        let dependency_result = DependencyResolver::new(self.indexes).scan_fn(node);
        validators::item::check_circular_dependencies(ref_, dependency_result, &mut self.context)?;
        validators::item::check_prelude_location(ref_, compilerimpl_span, &mut self.context)?;
        self.run_with_param_constness(ParamConstness::ExplicitOnly, |self_| {
            self_.validate_params(&node.params, compilerimpl_span.is_some())?;
            self_.validate_fn_return_type(node)?;
            Ok(())
        })?;
        validators::item::check_unary_operator_fn(node, &mut self.context, self.indexes)?;
        validators::item::check_binary_operator_fn(node, &mut self.context, self.indexes)?;
        self.validate_body(node)?;
        self.validate_fn_name(node);
        validators::item::check_usage(
            ref_,
            &mut self.context,
            &mut self.key_renderer,
            self.indexes,
        );
        Ok(())
    }

    fn validate_fn_name(&mut self, node: &FnDefinition) {
        let allowed_cases = self.fn_allowed_cases(node);
        validators::ident::check_case(node.name_span, allowed_cases, &mut self.context);
        validators::ident::check_char_count(node.name_span, &mut self.context);
    }

    fn validate_fn_return_type(&mut self, node: &FnDefinition) -> Result<(), ValidateError> {
        let (Some(arrow_span), Some(return_type)) = (node.arrow_span, &node.return_type) else {
            return Ok(());
        };
        self.with_const_mark_span(Some(arrow_span), |self_| self_.validate_expr(return_type))?;
        validators::expr::check_types(
            return_type.span(),
            self.value_resolver.expr_type(return_type),
            None,
            Type::Struct(self.indexes.search_prelude_type("typeref")),
            &mut self.context,
        )?;
        Ok(())
    }

    fn validate_body(&mut self, node: &FnDefinition) -> Result<(), ValidateError> {
        let param_constness = if node.const_keyword_span.is_some() {
            ParamConstness::All
        } else {
            ParamConstness::ExplicitOnly
        };
        self.run_with_param_constness(param_constness, |self_| self_.validate_fn_statements(node))?;
        Ok(())
    }

    fn validate_fn_statements(&mut self, node: &FnDefinition) -> Result<(), ValidateError> {
        let FnBody::Statements(body) = &node.body else {
            return Ok(());
        };
        let mut is_error_detected = false;
        self.with_const_mark_span(node.const_keyword_span, |self_| {
            for (index, statement) in body.statements.iter().enumerate() {
                is_error_detected |= self_.validate_statement(statement).is_err();
                if let Statement::Return(return_) = statement {
                    let next_statement_span = body
                        .statements
                        .get(index + 1)
                        .map_or(body.body_end_span, Statement::span);
                    is_error_detected |= validators::statement::check_return_before_end(
                        return_.span,
                        next_statement_span,
                        index,
                        body.statements.len(),
                        &mut self_.context,
                    )
                    .is_err();
                }
            }
        });
        if is_error_detected {
            return Err(ValidateError);
        }
        if let Some(return_type) = &node.return_type {
            let previous_statement_span = body
                .statements
                .last()
                .map_or(body.body_start_span, Statement::span);
            let return_statement = validators::statement::check_missing_return(
                &body.statements,
                previous_statement_span,
                body.body_end_span,
                return_type.span(),
                &mut self.context,
            )?;
            validators::expr::check_types(
                return_statement.value.span(),
                self.value_resolver.expr_type(&return_statement.value),
                Some(return_type.span()),
                self.value_resolver.fn_type(node),
                &mut self.context,
            )?;
        } else {
            validators::statement::check_disallowed_return(
                &body.statements,
                node,
                &mut self.context,
            )?;
            validators::statement::check_empty_block(
                &body.statements,
                body.body_span,
                &mut self.context,
            );
        }
        Ok(())
    }

    fn validate_statement(&mut self, node: &Statement) -> Result<(), ValidateError> {
        match node {
            Statement::Return(statement) => self.validate_expr(&statement.value),
            Statement::Assignment(statement) => self.validate_assignment_statement(statement),
        }
    }

    fn validate_assignment_statement(
        &mut self,
        node: &AssignmentStatement,
    ) -> Result<(), ValidateError> {
        let assigned_result = self.validate_assignment_statement_assigned(node);
        let value_result = self.validate_assignment_statement_value(node);
        assigned_result.and(value_result)
    }

    fn validate_assignment_statement_assigned(
        &mut self,
        node: &AssignmentStatement,
    ) -> Result<(), ValidateError> {
        self.validate_expr(&node.assigned)?;
        validators::expr::check_ref(&node.assigned, &mut self.context, self.indexes);
        Ok(())
    }

    fn validate_assignment_statement_value(
        &mut self,
        node: &AssignmentStatement,
    ) -> Result<(), ValidateError> {
        self.validate_expr(&node.value)?;
        validators::expr::check_types(
            node.value.span(),
            self.value_resolver.expr_type(&node.value),
            Some(node.assigned.span()),
            self.value_resolver.expr_type(&node.assigned),
            &mut self.context,
        )?;
        Ok(())
    }
}
