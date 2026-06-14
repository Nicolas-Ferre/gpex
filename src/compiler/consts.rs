use crate::compiler::indexing::item_ref::ItemRef;

#[derive(Debug)]
pub(crate) struct ConstChecker {
    pub(crate) param_constness: ParamConstness,
}

impl ConstChecker {
    pub(crate) fn new() -> Self {
        Self {
            param_constness: ParamConstness::ExplicitOnly,
        }
    }

    pub(crate) fn is_item_const(&self, node: ItemRef<'_>) -> bool {
        match node {
            ItemRef::Var(_) => false,
            ItemRef::Const(_) | ItemRef::Struct(_) => true,
            ItemRef::Fn(node) => node.const_keyword_span.is_some(),
            ItemRef::Param(node) => match self.param_constness {
                ParamConstness::ExplicitOnly => node.const_mark_span().is_some(),
                ParamConstness::All => true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ParamConstness {
    ExplicitOnly,
    All,
}
