use crate::compiler::consts::ConstChecker;
use crate::compiler::indexing::indexer::FN_CALL_SEARCH_CONFIG;
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::actions::RepeatDefinition;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::parsing::statements::Statement;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{SearchParams, Visibility};
use crate::utils::parsing::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyType {
    CycleDetection,
    Transpilation,
}

#[derive(Debug)]
pub(crate) struct DependencyResolver<'item, 'index> {
    pub(crate) dependencies: Dependencies<ItemRef<'item>>,
    type_: DependencyType,
    const_checker: ConstChecker<'item, 'index>,
    fn_config: FnConfig,
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> DependencyResolver<'item, 'index> {
    pub(crate) fn new(type_: DependencyType, indexes: &'index Indexes<'item>) -> Self {
        Self {
            dependencies: Dependencies::new(),
            type_,
            const_checker: ConstChecker::new(indexes),
            fn_config: FnConfig {
                is_fn_const: false,
                are_args_const: false,
            },
            indexes,
        }
    }

    pub(crate) fn scan_var(&mut self, node: &VarDefinition) -> Result<(), Vec<Span>> {
        let fn_config = FnConfig {
            is_fn_const: false,
            are_args_const: false,
        };
        self.run_with_fn_config(fn_config, |self_| self_.scan_expr(&node.default_value))
    }

    pub(crate) fn scan_const(&mut self, node: &ConstDefinition) -> Result<(), Vec<Span>> {
        let fn_config = FnConfig {
            is_fn_const: false,
            are_args_const: false,
        };
        self.run_with_fn_config(fn_config, |self_| self_.scan_expr(&node.value))
    }

    pub(crate) fn scan_fn(&mut self, node: &FnDefinition) -> Result<(), Vec<Span>> {
        let fn_config = FnConfig {
            is_fn_const: node.const_keyword_span.is_some(),
            are_args_const: self.fn_config.are_args_const,
        };
        self.run_with_fn_config(fn_config, |self_| {
            self_.scan_params(&node.params)?;
            if let Some(return_type) = &node.return_type {
                self_.scan_expr(return_type)?;
            }
            if let FnBody::Statements(body) = &node.body {
                for statement in &body.statements {
                    self_.scan_statement(statement)?;
                }
            }
            Ok(())
        })
    }

    pub(crate) fn scan_repeat(&mut self, node: &RepeatDefinition) -> Result<(), Vec<Span>> {
        self.scan_call(&node.call)
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
        if self.type_ == DependencyType::Transpilation && self.const_checker.is_expr_const(node) {
            return Ok(());
        }
        match node {
            Expr::F32Literal(_)
            | Expr::U32Literal(_)
            | Expr::I32Literal(_)
            | Expr::BoolLiteral(_) => Ok(()),
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
            let fn_config = FnConfig {
                is_fn_const: false,
                are_args_const: node
                    .args
                    .iter()
                    .all(|arg| self.const_checker.is_expr_const(arg)),
            };
            self.run_with_fn_config(fn_config, |self_| self_.scan_item(source, node.span))
        } else {
            // Covers case where there is function circular dependency from their signature.
            for source in self.search_not_indexed_call_source(node) {
                let fn_config = FnConfig {
                    is_fn_const: false,
                    are_args_const: false,
                };
                self.run_with_fn_config(fn_config, |self_| self_.scan_item(source, node.span))?;
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

    fn search_not_indexed_call_source(&self, node: &Call) -> Vec<ItemRef<'item>> {
        let search_params = SearchParams {
            key: &node.key(),
            location: node,
            imports: &self.indexes.imports,
            config: FN_CALL_SEARCH_CONFIG,
        };
        self.indexes
            .items
            .search(search_params, Visibility::Enforced)
            .filter(|item| matches!(item, ItemRef::Fn(_)))
            .collect()
    }

    fn run_with_fn_config<O>(
        &mut self,
        config: FnConfig,
        callback: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let previous_fn_config = self.fn_config;
        self.fn_config = config;
        self.const_checker
            .set_is_in_const_fn(self.fn_config.is_fn_const && self.fn_config.are_args_const);
        let output = callback(self);
        self.fn_config = previous_fn_config;
        self.const_checker
            .set_is_in_const_fn(self.fn_config.is_fn_const && self.fn_config.are_args_const);
        output
    }
}

#[derive(Debug, Clone, Copy)]
struct FnConfig {
    is_fn_const: bool,
    are_args_const: bool,
}
