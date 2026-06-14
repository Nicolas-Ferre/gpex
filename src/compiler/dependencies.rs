use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::parsing::statements::Statement;
use crate::utils::dependencies::Dependencies;
use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct DependencyResolver<'item, 'index> {
    pub(crate) dependencies: Dependencies<ItemRef<'item>>,
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> DependencyResolver<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            dependencies: Dependencies::new(),
            indexes,
        }
    }

    pub(crate) fn scan_var(&mut self, node: &VarDefinition) -> Result<(), Vec<Span>> {
        self.scan_expr(&node.default_value)
    }

    pub(crate) fn scan_const(&mut self, node: &ConstDefinition) -> Result<(), Vec<Span>> {
        self.scan_expr(&node.value)
    }

    pub(crate) fn scan_fn(&mut self, node: &FnDefinition) -> Result<(), Vec<Span>> {
        self.scan_params(&node.params)?;
        if let Some(return_type) = &node.return_type {
            self.scan_expr(return_type)?;
        }
        if let FnBody::Statements(body) = &node.body {
            for statement in &body.statements {
                self.scan_statement(statement)?;
            }
        }
        Ok(())
    }

    fn scan_statement(&mut self, node: &Statement) -> Result<(), Vec<Span>> {
        match node {
            Statement::Return(child) => self.scan_expr(&child.value)?,
            Statement::Assignment(child) => {
                self.scan_expr(&child.assigned)?;
                self.scan_expr(&child.value)?;
            }
        }
        Ok(())
    }

    fn scan_expr(&mut self, node: &Expr) -> Result<(), Vec<Span>> {
        match node {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_)
            | Expr::Wildcard(_) => Ok(()),
            Expr::Call(node) => self.scan_call(node),
            Expr::Ident(node) => self.scan_ident(node),
        }
    }

    fn scan_ident(&mut self, node: &Ident) -> Result<(), Vec<Span>> {
        if let Some(&source) = self.indexes.sources.get(&node.id) {
            self.scan_item(source, node.span)
        } else {
            Ok(())
        }
    }

    fn scan_call(&mut self, node: &Call) -> Result<(), Vec<Span>> {
        for arg in &node.args {
            self.scan_expr(arg)?; // no-fn-check (recursivity)
        }
        if let Some(&source) = self.indexes.sources.get(&node.id) {
            self.scan_item(source, node.span)
        } else {
            // Covers case where there is function circular dependency from their signature.
            // As call source resolution is not done in this case, candidates are followed instead.
            for &source in self
                .indexes
                .candidate_sources
                .get(&node.id)
                .into_iter()
                .flatten()
            {
                self.scan_item(source, node.span)?;
            }
            Ok(())
        }
    }

    fn scan_item(&mut self, node: ItemRef<'item>, ref_span: Span) -> Result<(), Vec<Span>> {
        self.dependencies.enter_item(ref_span, node)?;
        match node {
            ItemRef::Var(child) => self.scan_var(child)?,
            ItemRef::Const(child) => self.scan_const(child)?,
            ItemRef::Struct(_) => (),
            ItemRef::Fn(child) => self.scan_fn(child)?,
            ItemRef::Param(child) => self.scan_param(child)?,
        }
        self.dependencies.exit_item();
        Ok(())
    }

    fn scan_params(&mut self, node: &ParamGroup) -> Result<(), Vec<Span>> {
        for param in &node.params {
            self.scan_param(param)?;
        }
        Ok(())
    }

    fn scan_param(&mut self, node: &Param) -> Result<(), Vec<Span>> {
        self.scan_expr(&node.type_) // no-fn-check (recursivity)
    }
}
