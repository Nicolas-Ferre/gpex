use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::parsing::statements::{AssignmentStatement, ReturnStatement, Statement};
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::transpilation::{SpecializedFn, Transpiler};
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types::Type;
use std::fmt::Write;

impl<'item> Transpiler<'item, '_> {
    pub(crate) fn transpile_specialized_fn(&mut self, node: SpecializedFn<'item>, fn_index: usize) {
        if !self.transpiled_specialized_fn_indexes.insert(fn_index) {
            return;
        }
        self.value_resolver.enter_scope();
        let source = node.fn_;
        let id = source.id;
        _ = write!(self.shader, "fn _{id}_{fn_index}");
        self.transpile_params(
            &source.params,
            node.const_param_values.into_iter(),
            node.wildcard_param_types.into_iter(),
        );
        if let Some(return_type) = self.value_resolver.fn_type(source).struct_ref() {
            let return_type_name = Self::transpile_type_name(return_type);
            _ = write!(self.shader, " -> {return_type_name} {{ ");
        } else {
            _ = write!(self.shader, " {{ ");
        }
        self.transpile_mut_param_definitions(&source.params);
        for statement in &node.fn_body.statements {
            self.transpile_statement(statement);
        }
        _ = write!(self.shader, " }}");
        self.value_resolver.exit_scope();
    }

    pub(crate) fn transpile_var_init(&mut self, node: &VarDefinition) {
        self.transpile_var_ref(node);
        self.shader += " = ";
        self.transpile_expr(&node.default_value);
        self.shader += "; ";
    }

    pub(crate) fn transpile_var_as_struct_field(&mut self, node: &VarDefinition) {
        let type_ = self
            .value_resolver
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

    fn transpile_params(
        &mut self,
        node: &'item ParamGroup,
        mut const_param_values: impl Iterator<Item = ConstValue<'item>>,
        mut wildcard_param_types: impl Iterator<Item = &'item StructDefinition>,
    ) {
        self.shader += "(";
        for param in &node.params {
            self.resolve_param_wildcard_type(param, &mut wildcard_param_types);
            if param.const_mark_span().is_some() {
                self.resolve_const_param_value(param, &mut const_param_values);
            } else {
                self.transpile_param(param);
                self.shader += ", ";
            }
        }
        self.shader += ")";
    }

    fn transpile_param(&mut self, node: &'item Param) {
        let id = node.id;
        let type_ = self
            .value_resolver
            .param_type(node)
            .struct_ref()
            .unwrap_or_else(|| unreachable!("parameter type should be validated before"));
        let type_name = Self::transpile_type_name(type_);
        _ = write!(self.shader, "_{id}_const: {type_name}");
    }

    fn transpile_mut_param_definitions(&mut self, node: &ParamGroup) {
        for param in &node.params {
            if param.const_mark_span().is_none() {
                self.transpile_mut_param_definition(param);
            }
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
            _ => unreachable!("not implemented `{}` GPU type", type_.name),
        }
    }

    fn resolve_const_param_value(
        &mut self,
        param: &Param,
        const_param_values: &mut impl Iterator<Item = ConstValue<'item>>,
    ) {
        let value = const_param_values
            .next()
            .unwrap_or_else(|| unreachable!("mismatching number of const params"));
        self.value_resolver.add_value(param.id, value);
    }

    fn resolve_param_wildcard_type(
        &mut self,
        param: &Param,
        wildcard_param_types: &mut impl Iterator<Item = &'item StructDefinition>,
    ) {
        if !matches!(param.type_, Expr::Wildcard(_)) {
            return;
        }
        let type_ = wildcard_param_types
            .next()
            .unwrap_or_else(|| unreachable!("mismatching number of wildcard params"));
        self.value_resolver.add_type(param.id, Type::Struct(type_));
    }
}
