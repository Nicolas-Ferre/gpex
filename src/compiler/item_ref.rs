use crate::compiler::consts;
use crate::compiler::consts::ConstValue;
use crate::compiler::key_rendering;
use crate::compiler::parsing::exprs::calls::Arg;
use crate::compiler::parsing::items::fns::{FnDefinition, IntrinsicFn};
use crate::compiler::parsing::items::params::{Param, ParamGroup};
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::parsing::items::vars::{ConstDefinition, VarDefinition};
use crate::compiler::state::State;
use crate::compiler::types::{self, Type};
use crate::utils::indexing::{ItemNodeRef, NodeRef};
use crate::utils::parsing::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ItemRef<'item> {
    Var(&'item VarDefinition),
    Const(&'item ConstDefinition),
    Struct(&'item StructDefinition),
    Fn(&'item FnDefinition),
    Param(&'item Param),
}

impl NodeRef for ItemRef<'_> {
    fn file_index(&self) -> usize {
        match self {
            Self::Var(var) => var.name_span.file_index,
            Self::Const(const_) => const_.name_span.file_index,
            Self::Struct(struct_) => struct_.name_span.file_index,
            Self::Fn(fn_) => fn_.name_span.file_index,
            Self::Param(param) => param.name_span.file_index,
        }
    }

    fn id(&self) -> u64 {
        match self {
            Self::Var(var) => var.id,
            Self::Const(const_) => const_.id,
            Self::Struct(struct_) => struct_.id,
            Self::Fn(fn_) => fn_.id,
            Self::Param(param) => param.id,
        }
    }

    fn scope(&self) -> &[u64] {
        match self {
            Self::Var(var) => &var.scope,
            Self::Const(const_) => &const_.scope,
            Self::Struct(struct_) => &struct_.scope,
            Self::Fn(fn_) => &fn_.scope,
            Self::Param(param) => &param.scope,
        }
    }
}

impl ItemNodeRef for ItemRef<'_> {
    fn is_pub(&self) -> bool {
        match self {
            Self::Var(var) => var.pub_keyword_span.is_some(),
            Self::Const(const_) => const_.pub_keyword_span.is_some(),
            Self::Struct(struct_) => struct_.pub_keyword_span.is_some(),
            Self::Fn(fn_) => fn_.pub_keyword_span.is_some(),
            Self::Param(_) => false,
        }
    }

    fn key(&self) -> String {
        match self {
            Self::Var(var) => var.name.clone(),
            Self::Const(const_) => const_.name.clone(),
            Self::Struct(struct_) => struct_.name.clone(),
            Self::Fn(fn_) => fn_.key(),
            Self::Param(param) => param.name.clone(),
        }
    }
}

impl<'item> ItemRef<'item> {
    pub(crate) fn name_span(self) -> Span {
        match self {
            Self::Var(var) => var.name_span,
            Self::Const(const_) => const_.name_span,
            Self::Struct(struct_) => struct_.name_span,
            Self::Fn(fn_) => fn_.name_span,
            Self::Param(param) => param.name_span,
        }
    }

    pub(crate) fn signature_span_with_return(self) -> Span {
        match self {
            Self::Fn(fn_) => fn_.signature_span_with_return,
            // coverage: off (only functions can be called)
            Self::Var(_) | Self::Const(_) | Self::Struct(_) | Self::Param(_) => {
                unreachable!("only functions can be called")
            } // coverage: on
        }
    }

    pub(crate) fn args_match(self, args: &[Arg], state: &State<'item>) -> ArgsMatch {
        let params = self.params();
        state.in_scope(|state| {
            params
                .params
                .iter()
                .zip(args)
                .try_fold(ArgsMatch::Matching, |result, (param, arg)| {
                    let (param_type, arg_type) = types::bind_param_to_arg(param, arg, state);
                    let param_match = Self::param_match(param_type, arg_type, param, state);
                    ArgsMatch::try_combine(result, param_match)
                })
                .unwrap_or(ArgsMatch::NotMatching)
        })
    }

    pub(crate) fn params(self) -> &'item ParamGroup {
        match self {
            ItemRef::Fn(item) => &item.params,
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("only functions can have parameters")
            }
        }
    }

    pub(crate) fn displayed_key(self, state: &State<'item>) -> String {
        match self {
            ItemRef::Fn(item) => key_rendering::fn_key(item, state)
                .unwrap_or_else(|_| unreachable!("function should be validated before")),
            ItemRef::Var(item) => item.name.clone(),
            ItemRef::Const(item) => item.name.clone(),
            ItemRef::Param(item) => item.name.clone(),
            ItemRef::Struct(_) => unreachable!("structs are not yet validated"),
        }
    }

    pub(crate) fn is_param_constness_ignored(self) -> bool {
        matches!(self, ItemRef::Fn(fn_) if fn_.intrinsic() == Some(IntrinsicFn::Typeof))
    }

    pub(crate) fn is_const(self, are_params_const: bool) -> bool {
        match self {
            Self::Var(_) => false,
            Self::Const(_) | Self::Struct(_) => true,
            Self::Fn(fn_) => fn_.const_keyword_span.is_some(),
            Self::Param(param) => are_params_const || param.const_mark_span().is_some(),
        }
    }

    fn param_match(
        param_type: Type<'_>,
        arg_type: Type<'_>,
        param: &Param,
        state: &State<'item>,
    ) -> ArgsMatch {
        let arg_match = Self::arg_match(param_type, arg_type);
        if arg_match == ArgsMatch::NotMatching {
            return ArgsMatch::NotMatching;
        }
        ArgsMatch::combine([arg_match, Self::requirement_match(param, state)])
    }

    fn arg_match(param_type: Type<'_>, arg_type: Type<'_>) -> ArgsMatch {
        if matches!(param_type, Type::Wildcard(_)) || param_type == arg_type {
            ArgsMatch::Matching
        } else if matches!(param_type, Type::Unknown) || matches!(arg_type, Type::Unknown) {
            ArgsMatch::Unknown
        } else {
            ArgsMatch::NotMatching
        }
    }

    fn requirement_match(param: &Param, state: &State<'item>) -> ArgsMatch {
        if let Some(requirement) = &param.requirement {
            match consts::expr_value(&requirement.condition, state) {
                ConstValue::Bool(true) => ArgsMatch::Matching,
                ConstValue::Unknown => ArgsMatch::Unknown,
                ConstValue::TypeRef(_)
                | ConstValue::Param(_)
                | ConstValue::WildcardType(_)
                | ConstValue::I32(_)
                | ConstValue::U32(_)
                | ConstValue::F32(_)
                | ConstValue::Bool(false)
                | ConstValue::RuntimeValue => ArgsMatch::NotMatching,
            }
        } else {
            ArgsMatch::Matching
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArgsMatch {
    Matching,
    NotMatching,
    Unknown,
}

impl ArgsMatch {
    fn try_combine(result: Self, next: Self) -> Result<Self, Self> {
        match next {
            Self::NotMatching => Err(Self::NotMatching),
            Self::Matching | Self::Unknown => Ok(Self::combine([result, next])),
        }
    }

    fn combine(matches: [Self; 2]) -> Self {
        if matches.contains(&Self::NotMatching) {
            Self::NotMatching
        } else if matches.contains(&Self::Unknown) {
            Self::Unknown
        } else {
            Self::Matching
        }
    }
}
