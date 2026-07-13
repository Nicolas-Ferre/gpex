use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{CompilerImpl, FnDefinition};
use crate::compiler::transpilation::Transpiler;
use crate::compiler::values::types::Type;
use std::fmt::Write;

impl Transpiler<'_, '_> {
    pub(super) fn transpile_compilerimpl_fn_call(&mut self, node: &Call, source: &FnDefinition) {
        match source.compilerimpl() {
            Some(
                compilerimpl @ (CompilerImpl::Add
                | CompilerImpl::Sub
                | CompilerImpl::Mul
                | CompilerImpl::Div
                | CompilerImpl::Mod
                | CompilerImpl::Eq
                | CompilerImpl::Ne
                | CompilerImpl::Lt
                | CompilerImpl::Le
                | CompilerImpl::Gt
                | CompilerImpl::Ge
                | CompilerImpl::And
                | CompilerImpl::Or),
            ) => self.transpile_compilerimpl_fn_call_binary(node, compilerimpl),
            Some(compilerimpl @ (CompilerImpl::Neg | CompilerImpl::Not)) => {
                self.transpile_compilerimpl_fn_call_unary(node, compilerimpl);
            }
            Some(CompilerImpl::MulAdd) => self.transpile_compilerimpl_fn_call_mul_add(node),
            Some(CompilerImpl::Typeof | CompilerImpl::Sizeof) | None => {
                unreachable!("not implemented `{}` GPU function", source.name)
            }
        }
    }

    // TODO: this function is way to big
    #[allow(clippy::wildcard_enum_match_arm)]
    fn transpile_compilerimpl_fn_call_binary(&mut self, node: &Call, compilerimpl: CompilerImpl) {
        let is_typeref_comparison = matches!(
            self.value_resolver.expr_type(&node.args[0].value),
            Type::Struct(type_) if type_.name == "typeref"
        );
        if is_typeref_comparison {
            if compilerimpl == CompilerImpl::Ne {
                self.shader += "u32(!all(";
            } else {
                self.shader += "u32(all(";
            }
            self.transpile_expr(&node.args[0].value);
            self.shader += " == ";
            self.transpile_expr(&node.args[1].value);
            self.shader += "))";
            return;
        }
        let operator = match compilerimpl {
            CompilerImpl::Add => "+",
            CompilerImpl::Sub => "-",
            CompilerImpl::Mul => "*",
            CompilerImpl::Div => "/",
            CompilerImpl::Mod => "%",
            CompilerImpl::Eq => "==",
            CompilerImpl::Ne => "!=",
            CompilerImpl::Lt => "<",
            CompilerImpl::Le => "<=",
            CompilerImpl::Gt => ">",
            CompilerImpl::Ge => ">=",
            CompilerImpl::And => "&&",
            CompilerImpl::Or => "||",
            _ => unreachable!("invalid binary compiler implementation"),
        };
        let is_comparison = matches!(
            compilerimpl,
            CompilerImpl::Eq
                | CompilerImpl::Ne
                | CompilerImpl::Lt
                | CompilerImpl::Le
                | CompilerImpl::Gt
                | CompilerImpl::Ge
        );
        let is_boolean = matches!(compilerimpl, CompilerImpl::And | CompilerImpl::Or);
        if is_comparison || is_boolean {
            self.shader += "u32(";
        }
        self.shader += "(";
        if is_boolean {
            self.shader += "(";
        }
        self.transpile_expr(&node.args[0].value);
        if is_boolean {
            self.shader += " != u32(0))";
        }
        _ = write!(self.shader, " {operator} ");
        if is_boolean {
            self.shader += "(";
        }
        self.transpile_expr(&node.args[1].value);
        if is_boolean {
            self.shader += " != u32(0))";
        }
        self.shader += ")";
        if is_comparison || is_boolean {
            self.shader += ")";
        }
    }

    fn transpile_compilerimpl_fn_call_unary(&mut self, node: &Call, compilerimpl: CompilerImpl) {
        if compilerimpl == CompilerImpl::Not {
            self.shader += "u32(";
            self.transpile_expr(&node.args[0].value);
            self.shader += " == u32(0))";
        } else {
            self.shader += "(-";
            self.transpile_expr(&node.args[0].value);
            self.shader += ")";
        }
    }

    fn transpile_compilerimpl_fn_call_mul_add(&mut self, node: &Call) {
        self.shader += "fma(";
        for arg in &node.args {
            self.transpile_expr(&arg.value);
            self.shader += ", ";
        }
        self.shader += ")";
    }
}
