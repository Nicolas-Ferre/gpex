mod facts;

use self::facts::{TypeFact, TypeFactOperand};
use crate::compiler::consts::{self, ConstValue};
use crate::compiler::indexing::{IndexState, exprs};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::queries;
use crate::compiler::queries::params;
use crate::compiler::state::{State, TypeFacts};
use crate::compiler::types::{self, Type};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) enum LogicalTypeNarrowing {
    And,
    Or,
}

impl LogicalTypeNarrowing {
    pub(super) fn logical_operator(self) -> BinaryIntrinsicFn {
        match self {
            Self::And => BinaryIntrinsicFn::And,
            Self::Or => BinaryIntrinsicFn::Or,
        }
    }

    pub(super) fn comparison_operator(self) -> BinaryIntrinsicFn {
        match self {
            Self::And => BinaryIntrinsicFn::Eq,
            Self::Or => BinaryIntrinsicFn::Ne,
        }
    }
}

#[derive(Default)]
pub(super) struct TypeNarrowingState<'item> {
    type_facts: HashMap<u64, Rc<TypeFacts<'item>>>,
}

impl<'item> TypeNarrowingState<'item> {
    fn type_facts(&self, param: &Param) -> Rc<TypeFacts<'item>> {
        self.type_facts.get(&param.id).cloned().unwrap_or_default()
    }
}

pub(super) fn index_logical_operation_args<'item>(
    call: &'item Call,
    narrowing: LogicalTypeNarrowing,
    state: &mut IndexState<'_, 'item>,
) {
    exprs::index_expr(&call.args[0].value, state);
    let mut facts = vec![];
    collect_type_facts(&call.args[0].value, narrowing, state.inner, &mut facts);
    let previous_facts = state.type_narrowing.type_facts.clone();
    for fact in facts {
        fact.add(state);
    }
    exprs::index_expr(&call.args[1].value, state);
    state.type_narrowing.type_facts = previous_facts;
}

pub(super) fn index_ident<'item>(
    ident: &Ident,
    source: ItemRef<'item>,
    state: &mut IndexState<'_, 'item>,
) {
    if let ItemRef::Param(param) = source
        && let Some(fact_param) = type_fact_param(param, state.inner)
        && let Some(facts) = state.type_narrowing.type_facts.get(&fact_param.id).cloned()
    {
        state.set_expr_type_facts(ident.id, facts);
    }
}

fn collect_type_facts<'item>(
    expr: &'item Expr,
    narrowing: LogicalTypeNarrowing,
    state: &State<'item>,
    facts: &mut Vec<TypeFact<'item>>,
) {
    match expr {
        Expr::Parenthesized(parenthesized) => {
            collect_type_facts(&parenthesized.value, narrowing, state, facts);
        }
        Expr::Call(call) if is_logical_operator(call, narrowing, state) => {
            for arg in &call.args {
                collect_type_facts(&arg.value, narrowing, state, facts);
            }
        }
        Expr::Call(call) if is_comparison_operator(call, narrowing, state) => {
            facts.extend(comparison_type_fact(call, state));
        }
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_)
        | Expr::Ident(_) => {}
    }
}

fn is_logical_operator(call: &Call, narrowing: LogicalTypeNarrowing, state: &State<'_>) -> bool {
    queries::calls::is_binary_intrinsic(call, narrowing.logical_operator(), state)
}

fn is_comparison_operator(call: &Call, narrowing: LogicalTypeNarrowing, state: &State<'_>) -> bool {
    queries::calls::is_binary_intrinsic(call, narrowing.comparison_operator(), state)
}

fn comparison_type_fact<'item>(call: &'item Call, state: &State<'item>) -> Option<TypeFact<'item>> {
    let left_arg = &call.args[0].value;
    let right_arg = &call.args[1].value;
    match (
        type_fact_operand_subject(left_arg, state),
        type_fact_operand_subject(right_arg, state),
    ) {
        (Some(left), Some(right)) => Some(TypeFact {
            operands: [left, right],
            condition_node_id: call.id,
            subject_spans: [Some(left_arg.span()), Some(right_arg.span())],
        }),
        (Some(left), None) => concrete_type_fact(call.id, left, left_arg, right_arg, state),
        (None, Some(right)) => concrete_type_fact(call.id, right, right_arg, left_arg, state),
        (None, None) => None,
    }
}

fn concrete_type_fact<'item>(
    condition_node_id: u64,
    subject: TypeFactOperand<'item>,
    subject_expr: &Expr,
    type_expr: &Expr,
    state: &State<'item>,
) -> Option<TypeFact<'item>> {
    if let ConstValue::TypeRef(type_) = consts::expr_value(type_expr, state) {
        Some(TypeFact {
            operands: [subject, TypeFactOperand::Concrete(type_)],
            condition_node_id,
            subject_spans: [Some(subject_expr.span()), None],
        })
    } else {
        None
    }
}

fn type_fact_operand_subject<'item>(
    operand: &Expr,
    state: &State<'item>,
) -> Option<TypeFactOperand<'item>> {
    if let Some(param) = typeof_param(operand, state) {
        return Some(TypeFactOperand::from_param(param, state));
    }
    if let Expr::Ident(ident) = unwrap_parentheses(operand)
        && let Some(ItemRef::Param(param)) = state.sources.get(&ident.id).copied()
        && params::is_const_typeref(param, state)
    {
        Some(TypeFactOperand::Param(param))
    } else {
        None
    }
}

fn type_fact_param<'item>(param: &'item Param, state: &State<'item>) -> Option<&'item Param> {
    match types::param_type(param, state) {
        Type::Param(type_param) => Some(type_param),
        Type::Wildcard(_) => Some(param),
        Type::Struct(_) | Type::NoReturn | Type::Unknown => None,
    }
}

fn typeof_param<'item>(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
    if let Expr::Call(typeof_call) = unwrap_parentheses(expr)
        && queries::calls::is_intrinsic(typeof_call, IntrinsicFn::Typeof, state)
        && let Expr::Ident(ident) = unwrap_parentheses(&typeof_call.args[0].value)
        && let Some(ItemRef::Param(param)) = state.sources.get(&ident.id).copied()
    {
        Some(param)
    } else {
        None
    }
}

fn unwrap_parentheses(expr: &Expr) -> &Expr {
    match expr {
        Expr::Parenthesized(parenthesized) => unwrap_parentheses(&parenthesized.value),
        Expr::F32Literal(_)
        | Expr::U32Literal(_)
        | Expr::I32Literal(_)
        | Expr::BoolLiteral(_)
        | Expr::Wildcard(_)
        | Expr::Call(_)
        | Expr::Ident(_) => expr,
    }
}
