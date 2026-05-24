use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::items::fns::{FnBody, FnDefinition};
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, ReturnStatement, Statement};
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::transpilation::Transpiler;
use std::fmt::Write;

impl Transpiler<'_, '_> {
    pub(crate) fn transpile_item(&mut self, node: ItemRef<'_>) {
        match node {
            ItemRef::Fn(item) => self.transpile_fn(item),
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => (),
        }
    }

    pub(crate) fn transpile_fn(&mut self, node: &FnDefinition) {
        let FnBody::Statements(body) = &node.body else {
            return;
        };
        let id = node.id;
        _ = write!(self.shader, "fn _{id}");
        self.transpile_params(&node.params);
        if let Some(return_type) = self.type_resolver.fn_type(node).struct_ref() {
            let return_type_name = Self::transpile_type_name(return_type);
            _ = write!(self.shader, " -> {return_type_name} {{ ");
        } else {
            _ = write!(self.shader, " {{ ");
        }
        self.transpile_mut_param_definitions(&node.params);
        for statement in &body.statements {
            self.transpile_statement(statement);
        }
        _ = write!(self.shader, " }}");
    }

    pub(crate) fn transpile_var_init(&mut self, node: &VarDefinition) {
        self.transpile_var_ref(node);
        self.shader += " = ";
        self.transpile_expr(&node.default_value);
        self.shader += "; ";
    }

    pub(crate) fn transpile_var_as_struct_field(&mut self, node: &VarDefinition) {
        let type_ = self
            .type_resolver
            .var_type(node)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("variable type should be validated before"));
        _ = write!(
            self.shader,
            "v{}: {}, ",
            node.id,
            Self::transpile_type_name(type_)
        );
    }

    fn transpile_statement(&mut self, node: &Statement) {
        match node {
            Statement::Return(node) => self.transpile_return_statement(node),
            Statement::Assignment(node) => self.transpile_assignment_statement(node),
        }
    }

    fn transpile_return_statement(&mut self, node: &ReturnStatement) {
        self.shader += "return ";
        self.transpile_expr(&node.value);
        self.shader += ";";
    }

    fn transpile_assignment_statement(&mut self, node: &AssignmentStatement) {
        self.transpile_expr(&node.assigned);
        self.shader += " = ";
        self.transpile_expr(&node.value);
        self.shader += ";";
    }

    fn transpile_params(&mut self, node: &ParamGroup) {
        self.shader += "(";
        for param in &node.params {
            self.transpile_param(param);
            self.shader += ", ";
        }
        self.shader += ")";
    }

    fn transpile_param(&mut self, node: &Param) {
        let id = node.id;
        let type_ = self
            .type_resolver
            .param_type(node)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("parameter type should be validated before"));
        let type_name = Self::transpile_type_name(type_);
        _ = write!(self.shader, "_{id}_const: {type_name}");
    }

    fn transpile_mut_param_definitions(&mut self, node: &ParamGroup) {
        for param in &node.params {
            self.transpile_mut_param_definition(param);
        }
    }

    fn transpile_mut_param_definition(&mut self, node: &Param) {
        let id = node.id;
        _ = write!(self.shader, "var _{id} = _{id}_const; ");
    }

    fn transpile_type_name(type_: &StructDefinition) -> &str {
        match (type_.name_span.file_index, type_.name.as_str()) {
            (PRELUDE_FILE_INDEX, "typeref") => "vec2<u32>",
            (PRELUDE_FILE_INDEX, "f32") => "f32",
            (PRELUDE_FILE_INDEX, "i32") => "i32",
            (PRELUDE_FILE_INDEX, "u32" | "bool") => "u32",
            _ => unreachable!("not implemented GPU type"),
        }
    }
}
