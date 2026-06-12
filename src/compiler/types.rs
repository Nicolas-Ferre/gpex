use crate::compiler::consts::{ConstResolver, ConstValue};
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use crate::utils::validation::ValidateError;
use derive_where::derive_where;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct TypeResolver<'item, 'index> {
    indexes: &'index Indexes<'item>,
    pub(crate) const_resolver: ConstResolver<'item, 'index>,
    scope_types: Vec<HashMap<u64, &'item StructDefinition>>,
}

impl<'item, 'index> TypeResolver<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self {
            indexes,
            const_resolver: ConstResolver::new(indexes),
            scope_types: vec![],
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scope_types.push(HashMap::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scope_types.pop();
    }

    pub(crate) fn add_type(&mut self, id: u64, type_: &'item StructDefinition) {
        self.scope_types
            .last_mut()
            .unwrap_or_else(|| unreachable!("wildcard parameter type scope should be entered"))
            .insert(id, type_);
    }

    pub(crate) fn var_type(&mut self, node: &VarDefinition) -> Type<'item> {
        self.expr_type(&node.default_value)
    }

    pub(crate) fn param_type(&mut self, node: &'item Param) -> Type<'item> {
        if matches!(node.type_, Expr::Wildcard(_)) {
            self.scope_types
                .last()
                .and_then(|types| types.get(&node.id).copied())
                .map_or(Type::Wildcard(node), Type::Struct)
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
        self.const_resolver.enter_scope();
        for (param, arg) in node.params.params.iter().zip(args) {
            let value = self.const_resolver.expr_value(arg);
            self.const_resolver.add_value(param.id, value);
        }
        let type_ = if let Some(return_type) = node.return_type.as_ref() {
            self.expr_as_type(return_type)
        } else {
            Type::NoReturn
        };
        self.const_resolver.exit_scope();
        type_
    }

    pub(crate) fn expr_as_type(&mut self, node: &Expr) -> Type<'item> {
        match self.const_resolver.expr_value(node) {
            ConstValue::TypeRef(type_) => Type::Struct(type_),
            ConstValue::Param(type_) => Type::Param(type_),
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

// TODO: create comparison function and replace all equalities of types
impl<'item> Type<'item> {
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
