use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::items::fns::{CompilerImpl, FnDefinition};
use crate::compiler::transpilation::Transpiler;

impl Transpiler<'_, '_> {
    pub(super) fn transpile_compilerimpl_fn_call(&mut self, node: &Call, source: &FnDefinition) {
        match source.compilerimpl() {
            Some(CompilerImpl::Add) => self.transpile_compilerimpl_fn_call_add(node),
            Some(CompilerImpl::Typeof | CompilerImpl::Sizeof) | None => {
                unreachable!("not implemented `{}` GPU function", source.name)
            }
        }
    }

    fn transpile_compilerimpl_fn_call_add(&mut self, node: &Call) {
        self.transpile_expr(&node.args[0]);
        self.shader += " + ";
        self.transpile_expr(&node.args[1]);
    }
}
