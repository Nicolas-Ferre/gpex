use crate::compiler::consts::ConstResolver;
use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::VarDefinition;
use derive_where::derive_where;

#[derive(Debug)]
pub(crate) struct TypeResolver<'item, 'index> {
    indexes: &'index Indexes<'item>,
}

impl<'item, 'index> TypeResolver<'item, 'index> {
    pub(crate) fn new(indexes: &'index Indexes<'item>) -> Self {
        Self { indexes }
    }

    pub(crate) fn var_type(&self, node: &VarDefinition) -> Type<'item> {
        self.expr_type(&node.default_value)
    }

    pub(crate) fn param_type(&self, node: &Param) -> Type<'item> {
        self.expr_as_type(&node.type_)
    }

    pub(crate) fn fn_type(&self, node: &FnDefinition) -> Type<'item> {
        if let Some(return_type) = node.return_type.as_ref() {
            self.expr_as_type(return_type)
        } else {
            Type::NoReturn
        }
    }

    pub(crate) fn expr_type(&self, node: &Expr) -> Type<'item> {
        match node {
            Expr::F32Literal(_) => Type::Struct(self.indexes.search_prelude_type("f32")),
            Expr::U32Literal(_) => Type::Struct(self.indexes.search_prelude_type("u32")),
            Expr::I32Literal(_) => Type::Struct(self.indexes.search_prelude_type("i32")),
            Expr::BoolLiteral(_) => Type::Struct(self.indexes.search_prelude_type("bool")),
            Expr::Call(node) => self.source_type(node.id),
            Expr::Ident(node) => self.source_type(node.id),
        }
    }

    pub(crate) fn expr_as_type(&self, node: &Expr) -> Type<'item> {
        let type_ = ConstResolver::new(self.indexes).expr_value(node).type_ref();
        if let Some(struct_) = type_ {
            Type::Struct(struct_)
        } else {
            Type::Unknown
        }
    }

    fn source_type(&self, node_id: u64) -> Type<'item> {
        match self.indexes.sources.get(&node_id) {
            Some(source) => self.item_type(*source),
            None => Type::Unknown,
        }
    }

    fn item_type(&self, node: ItemRef<'_>) -> Type<'item> {
        match node {
            ItemRef::Var(node) => self.var_type(node),
            ItemRef::Const(node) => self.expr_type(&node.value),
            ItemRef::Struct(_) => Type::Struct(self.indexes.search_prelude_type("typeref")),
            ItemRef::Fn(node) => {
                if let Some(return_type) = &node.return_type {
                    self.expr_as_type(return_type)
                } else {
                    Type::NoReturn
                }
            }
            ItemRef::Param(node) => self.param_type(node),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive_where(PartialEq)]
pub(crate) enum Type<'item> {
    Struct(&'item StructDefinition),
    NoReturn,
    #[derive_where(incomparable)]
    Unknown,
}

impl<'item> Type<'item> {
    pub(crate) fn name(self) -> &'item str {
        match self {
            Type::Struct(struct_) => &struct_.name,
            Type::NoReturn => unreachable!("no-type expression is not allowed as argument"),
            Type::Unknown => unreachable!("unknown-type expression is not allowed as argument"),
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
