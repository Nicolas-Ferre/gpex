use crate::compiler::indexing::item_ref::ItemRef;

#[derive(Debug)]
pub(crate) struct ConstChecker {
    pub(crate) location: ConstLocation,
}

impl ConstChecker {
    pub(crate) fn new() -> Self {
        Self {
            location: ConstLocation::Other,
        }
    }

    pub(crate) fn is_item_const(&self, node: ItemRef<'_>) -> bool {
        match node {
            ItemRef::Var(_) => false,
            ItemRef::Const(_) | ItemRef::Struct(_) => true,
            ItemRef::Fn(node) => node.const_keyword_span.is_some(),
            ItemRef::Param(node) => match self.location {
                ConstLocation::FnSignature | ConstLocation::ConstCallArg => {
                    node.const_mark_span().is_some()
                }
                ConstLocation::ConstFnBody => true,
                ConstLocation::Other => false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConstLocation {
    FnSignature,
    ConstFnBody,
    ConstCallArg,
    Other,
}
