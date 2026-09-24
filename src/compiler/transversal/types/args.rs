use super::Type;
use crate::compiler::transversal::ast::exprs::calls::Arg;
use crate::compiler::transversal::ast::item_ref::ItemRef;
use crate::compiler::transversal::ast::items::params::Param;
use crate::compiler::transversal::consts::{self, ConstValue};
use crate::compiler::transversal::state::State;

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

pub(crate) fn args_match<'item>(
    item: ItemRef<'item>,
    args: &[Arg],
    state: &State<'item>,
) -> ArgsMatch {
    let params = item.params();
    state.in_scope(|state| {
        params
            .params
            .iter()
            .zip(args)
            .try_fold(ArgsMatch::Matching, |result, (param, arg)| {
                let (param_type, arg_type) = super::bind_param_to_arg(param, arg, state);
                let param_match = param_match(param_type, arg_type, param, state);
                ArgsMatch::try_combine(result, param_match)
            })
            .unwrap_or(ArgsMatch::NotMatching)
    })
}

fn param_match(
    param_type: Type<'_>,
    arg_type: Type<'_>,
    param: &Param,
    state: &State<'_>,
) -> ArgsMatch {
    let arg_match = arg_match(param_type, arg_type);
    if arg_match == ArgsMatch::NotMatching {
        return ArgsMatch::NotMatching;
    }
    ArgsMatch::combine([arg_match, requirement_match(param, state)])
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

fn requirement_match(param: &Param, state: &State<'_>) -> ArgsMatch {
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
