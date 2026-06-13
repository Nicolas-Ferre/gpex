use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::compiler::values::{ConstValue, ValueResolver};
use crate::utils::validation::ValidateError;
use derive_where::derive_where;

impl<'item> ValueResolver<'item, '_> {
    pub(crate) fn add_type(&mut self, id: u64, type_: Type<'item>) {
        self.scopes
            .last_mut()
            .unwrap_or_else(|| unreachable!("wildcard parameter type scope should be entered"))
            .wildcard_types
            .insert(id, type_);
    }

    pub(crate) fn var_type(&mut self, node: &VarDefinition) -> Type<'item> {
        self.expr_type(&node.default_value)
    }

    pub(crate) fn param_type(&mut self, node: &'item Param) -> Type<'item> {
        if matches!(node.type_, Expr::Wildcard(_)) {
            self.scopes
                .last()
                .and_then(|scope| scope.wildcard_types.get(&node.id))
                .copied()
                .unwrap_or(Type::Wildcard(node))
        } else {
            self.expr_as_type(&node.type_)
        }
    }

    pub(crate) fn fn_type(&mut self, node: &FnDefinition) -> Type<'item> {
        if let Some(return_type) = node.return_type.as_ref() {
            self.expr_as_type(return_type)
        } else {
            Type::NoReturn
        }
    }

    pub(crate) fn expr_type(&mut self, node: &Expr) -> Type<'item> {
        match node {
            Expr::F32Literal(_) => Type::Struct(self.indexes.search_prelude_type("f32")),
            Expr::U32Literal(_) => Type::Struct(self.indexes.search_prelude_type("u32")),
            Expr::I32Literal(_) => Type::Struct(self.indexes.search_prelude_type("i32")),
            Expr::BoolLiteral(_) => Type::Struct(self.indexes.search_prelude_type("bool")),
            Expr::Wildcard(_) => Type::Unknown,
            Expr::Call(node) => self.source_type(node.id, &node.args),
            Expr::Ident(node) => self.source_type(node.id, &[]),
        }
    }

    fn source_type(&mut self, node_id: u64, args: &[Expr]) -> Type<'item> {
        match self.indexes.sources.get(&node_id) {
            Some(source) => self.item_type(*source, args),
            None => Type::Unknown,
        }
    }

    fn item_type(&mut self, node: ItemRef<'item>, args: &[Expr]) -> Type<'item> {
        match node {
            ItemRef::Var(node) => self.var_type(node),
            ItemRef::Const(node) => self.expr_type(&node.value),
            ItemRef::Struct(_) => Type::Struct(self.indexes.search_prelude_type("typeref")),
            ItemRef::Fn(node) => self.const_fn_type(node, args),
            ItemRef::Param(node) => self.param_type(node),
        }
    }

    pub(crate) fn const_fn_type(&mut self, node: &FnDefinition, args: &[Expr]) -> Type<'item> {
        self.run_scoped(|self_| {
            for (param, arg) in node.params.params.iter().zip(args) {
                if matches!(param.type_, Expr::Wildcard(_)) {
                    let arg_type = self_.expr_type(arg);
                    self_.add_type(param.id, arg_type);
                }
                let value = self_.expr_const_value(arg);
                self_.add_value(param.id, value);
            }
            if let Some(return_type) = node.return_type.as_ref() {
                self_.expr_as_type(return_type)
            } else {
                Type::NoReturn
            }
        })
    }

    pub(crate) fn expr_as_type(&mut self, node: &Expr) -> Type<'item> {
        match self.expr_const_value(node) {
            ConstValue::TypeRef(type_) => Type::Struct(type_),
            ConstValue::Param(type_) => Type::Param(type_),
            ConstValue::WildcardType(type_) => Type::Wildcard(type_),
            ConstValue::I32(_)
            | ConstValue::U32(_)
            | ConstValue::F32(_)
            | ConstValue::Bool(_)
            | ConstValue::Unknown
            | ConstValue::RuntimeValue => Type::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive_where(PartialEq)]
pub(crate) enum Type<'item> {
    Struct(&'item StructDefinition),
    Param(&'item Param),
    Wildcard(&'item Param),
    NoReturn,
    #[derive_where(incomparable)]
    Unknown,
}

impl<'item> Type<'item> {
    pub(crate) fn is_comparable(self) -> bool {
        matches!(self, Self::Struct(_) | Self::Param(_) | Self::Wildcard(_))
    }

    pub(crate) fn name(self) -> Result<String, ValidateError> {
        match self {
            Type::Struct(struct_) => Ok(struct_.name.clone()),
            Type::Param(param) => Ok(param.name.clone()),
            Type::Wildcard(param) => Ok(format!("typeof({})", param.name)),
            Type::NoReturn | Type::Unknown => Err(ValidateError),
        }
    }

    pub(crate) fn struct_ref(self) -> Option<&'item StructDefinition> {
        if let Self::Struct(struct_) = self {
            Some(struct_)
        } else {
            None
        }
    }
}
