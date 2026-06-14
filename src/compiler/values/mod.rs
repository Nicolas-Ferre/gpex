#![expect(clippy::multiple_inherent_impl)]

pub(crate) mod compilerimpl;
pub(crate) mod consts;
pub(crate) mod types;

use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::params::ParamGroup;
use crate::compiler::values::consts::ConstValue;
use crate::compiler::values::types::Type;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct ValueResolver<'item, 'index> {
    indexes: &'index Indexes<'item>,
    scopes: Vec<Scope<'item>>,
}

impl<'item, 'index> ValueResolver<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            indexes,
            scopes: vec![],
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn run_scoped<O>(&mut self, callback: impl FnOnce(&mut Self) -> O) -> O {
        self.enter_scope();
        let output = callback(self);
        self.exit_scope();
        output
    }

    pub(crate) fn bind_params_to_args(
        &mut self,
        params: &'item ParamGroup,
        args: &[Expr],
    ) -> impl Iterator<Item = (Type<'item>, Type<'item>)> {
        debug_assert_eq!(params.params.len(), args.len());
        params.params.iter().zip(args).map(|(param, arg)| {
            let param_type = self.param_type(param);
            let arg_type = self.expr_type(arg);
            if matches!(param.type_, Expr::Wildcard(_)) {
                self.add_type(param.id, arg_type);
            }
            if param.const_mark_span().is_some() {
                let value = self.expr_const_value(arg);
                self.add_value(param.id, value);
            }
            (param_type, arg_type)
        })
    }
}

#[derive(Debug, Default)]
struct Scope<'item> {
    const_values: HashMap<u64, ConstValue<'item>>,
    wildcard_types: HashMap<u64, Type<'item>>,
}
