use crate::compiler::dependencies::{DependencyResolver, DependencyType};
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, Statement};
use crate::compiler::types::Type;
use crate::compiler::validation::validators::ident::Case;
use crate::compiler::validation::{Validator, validators};
use crate::utils::parsing::span::Span;
use crate::utils::validation::ValidateError;

impl Validator<'_, '_> {
    pub(super) fn validate_fn(&mut self, node: &FnDefinition) -> Result<(), ValidateError> {
        self.run_with_fn_constness(node.const_keyword_span.is_some(), |self_| {
            let ref_ = ItemRef::Fn(node);
            let mut dependency_resolver =
                DependencyResolver::new(DependencyType::CycleDetection, self_.indexes);
            let dependency_result = dependency_resolver.scan_fn(node);
            validators::item::check_circular_dependencies(
                ref_,
                dependency_result,
                &mut self_.context,
            )?;
            self_.validate_params(&node.params)?;
            self_.validate_fn_return_type(node)?;
            self_.validate_fn_statements(node)?;
            self_.validate_fn_name(node);
            let Some(fn_key) = self_.key_renderer.fn_key(node) else {
                return Err(ValidateError);
            };
            validators::item::check_usage(ref_, &fn_key, &mut self_.context, self_.indexes);
            Ok(())
        })
    }

    fn run_with_fn_constness<O>(
        &mut self,
        is_in_const_fn: bool,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        self.const_checker.set_is_in_const_fn(is_in_const_fn);
        let result = callback(self);
        self.const_checker.set_is_in_const_fn(false);
        result
    }

    fn validate_fn_name(&mut self, node: &FnDefinition) {
        let typeref_type = self.indexes.search_prelude_type("typeref");
        let may_return_typeref = match self.type_resolver.fn_type(node) {
            Type::Struct(struct_ref) => struct_ref == typeref_type,
            Type::NoReturn => false,
            Type::Unknown => unreachable!("return type should be validated before"),
        };
        let allowed_cases: &[Case] = if may_return_typeref {
            &[Case::Snake, Case::Pascal]
        } else {
            &[Case::Snake]
        };
        validators::ident::check_case(node.name_span, allowed_cases, &mut self.context);
        validators::ident::check_char_count(node.name_span, &mut self.context);
    }

    fn validate_fn_return_type(&mut self, node: &FnDefinition) -> Result<(), ValidateError> {
        let (Some(arrow_span), Some(return_type)) = (node.arrow_span, &node.return_type) else {
            return Ok(());
        };
        self.validate_expr(return_type, Some(arrow_span))?;
        validators::expr::check_types(
            return_type.span(),
            self.type_resolver.expr_type(return_type),
            None,
            Type::Struct(self.indexes.search_prelude_type("typeref")),
            &mut self.context,
        )?;
        Ok(())
    }

    fn validate_fn_statements(&mut self, node: &FnDefinition) -> Result<(), ValidateError> {
        for (index, statement) in node.statements.iter().enumerate() {
            self.validate_statement(statement, node.const_keyword_span)?;
            if let Statement::Return(return_) = statement {
                validators::statement::check_return_before_end(
                    return_.span,
                    index,
                    node.statements.len(),
                    &mut self.context,
                )?;
            }
        }
        if let Some(return_type) = &node.return_type {
            let return_statement = validators::statement::check_missing_return(
                &node.statements,
                node.body_end_span,
                return_type.span(),
                &mut self.context,
            )?;
            validators::expr::check_types(
                return_statement.value.span(),
                self.type_resolver.expr_type(&return_statement.value),
                Some(return_type.span()),
                self.type_resolver.fn_type(node),
                &mut self.context,
            )?;
        } else {
            validators::statement::check_disallowed_return(
                &node.statements,
                node.name_span,
                &mut self.context,
            )?;
            validators::statement::check_empty_block(
                &node.statements,
                node.body_span,
                &mut self.context,
            );
        }
        Ok(())
    }

    fn validate_statement(
        &mut self,
        node: &Statement,
        const_mark_span: Option<Span>,
    ) -> Result<(), ValidateError> {
        let previous_const_mark_span = self.const_mark_span;
        self.const_mark_span = const_mark_span;
        let result = match node {
            Statement::Return(statement) => {
                self.validate_expr(&statement.value, self.const_mark_span)
            }
            Statement::Assignment(statement) => self.validate_assignment_statement(statement),
        };
        self.const_mark_span = previous_const_mark_span;
        result
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
        validators::expr::check_ref(&node.assigned, &mut self.context, self.indexes);
        self.validate_expr(&node.assigned, self.const_mark_span)?;
        Ok(())
    }

    fn validate_assignment_statement_value(
        &mut self,
        node: &AssignmentStatement,
    ) -> Result<(), ValidateError> {
        self.validate_expr(&node.value, self.const_mark_span)?;
        validators::expr::check_types(
            node.value.span(),
            self.type_resolver.expr_type(&node.value),
            Some(node.assigned.span()),
            self.type_resolver.expr_type(&node.assigned),
            &mut self.context,
        )?;
        Ok(())
    }
}
