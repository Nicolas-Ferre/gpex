mod facts;

use self::facts::{TypeFact, TypeFactOperand, TypeFactParam};
use crate::compiler::consts::{self, ConstValue};
use crate::compiler::indexing::{IndexState, exprs};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::queries;
use crate::compiler::state::{IntrinsicType, State, TypeFacts};
use crate::compiler::types;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) enum LogicalTypeNarrowing {
    And,
    Or,
}

impl LogicalTypeNarrowing {
    fn logical_operator(self) -> BinaryIntrinsicFn {
        match self {
            Self::And => BinaryIntrinsicFn::And,
            Self::Or => BinaryIntrinsicFn::Or,
        }
    }

    fn comparison_operator(self) -> BinaryIntrinsicFn {
        match self {
            Self::And => BinaryIntrinsicFn::Eq,
            Self::Or => BinaryIntrinsicFn::Ne,
        }
    }
}

#[derive(Default)]
pub(super) struct TypeNarrowingState<'item> {
    type_facts: HashMap<TypeFactParam<'item>, Rc<TypeFacts<'item>>>,
}

impl<'item> TypeNarrowingState<'item> {
    fn type_facts(&self, param: TypeFactParam<'item>) -> Rc<TypeFacts<'item>> {
        self.type_facts.get(&param).cloned().unwrap_or_default()
    }
}

struct ResolvedTypeFactOperand<'item> {
    operand: TypeFactOperand<'item>,
    is_subject: bool,
}

// TODO: can we replace it by a simple reset function? (in case we don't need to keep previous facts)
pub(super) fn with_empty_type_facts<'state, 'item, O>(
    state: &mut IndexState<'state, 'item>,
    callback: impl FnOnce(&mut IndexState<'state, 'item>) -> O,
) -> O {
    let previous_facts = mem::take(&mut state.type_narrowing.type_facts);
    let output = callback(state);
    state.type_narrowing.type_facts = previous_facts;
    output
}

pub(super) fn index_logical_operation_args<'item>(
    call: &'item Call,
    narrowing: LogicalTypeNarrowing,
    state: &mut IndexState<'_, 'item>,
) {
    exprs::index_expr(&call.args[0].value, state);
    let previous_facts = state.type_narrowing.type_facts.clone();
    add_expr_type_facts(&call.args[0].value, narrowing, state);
    exprs::index_expr(&call.args[1].value, state);
    state.type_narrowing.type_facts = previous_facts;
}

pub(super) fn add_expr_type_facts<'item>(
    expr: &'item Expr,
    narrowing: LogicalTypeNarrowing,
    state: &mut IndexState<'_, 'item>,
) {
    let mut facts = vec![];
    collect_type_facts(expr, narrowing, state.inner, &mut facts);
    for fact in facts {
        fact.add(state);
    }
}

pub(super) fn index_ident<'item>(
    ident: &Ident,
    source: ItemRef<'item>,
    state: &mut IndexState<'_, 'item>,
) {
    if let ItemRef::Param(param) = source
        && let Some(fact_param) = TypeFactOperand::from_param(param, state.inner).param()
        && let Some(facts) = state.type_narrowing.type_facts.get(&fact_param).cloned()
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
    let left_operand = resolve_type_fact_operand(left_arg, state)?;
    let right_operand = resolve_type_fact_operand(right_arg, state)?;
    if !left_operand.is_subject && !right_operand.is_subject {
        return None;
    }
    Some(TypeFact {
        operands: [left_operand.operand, right_operand.operand],
        condition_node_id: call.id,
        subject_spans: [
            left_operand.is_subject.then(|| left_arg.span()),
            right_operand.is_subject.then(|| right_arg.span()),
        ],
    })
}

fn resolve_type_fact_operand<'item>(
    operand: &Expr,
    state: &State<'item>,
) -> Option<ResolvedTypeFactOperand<'item>> {
    if let Some(param) = typeof_param(operand, state) {
        return Some(ResolvedTypeFactOperand {
            operand: TypeFactOperand::from_param(param, state),
            is_subject: true,
        });
    }
    let resolved_operand = match consts::expr_value(operand, state) {
        ConstValue::TypeRef(type_) => TypeFactOperand::Concrete(type_),
        ConstValue::Param(param) => TypeFactOperand::Param(TypeFactParam::ReferencedType(param)),
        ConstValue::WildcardType(param) => TypeFactOperand::Param(TypeFactParam::WildcardType(param)),
        ConstValue::I32(_)
        | ConstValue::U32(_)
        | ConstValue::F32(_)
        | ConstValue::Bool(_)
        | ConstValue::Unknown
        | ConstValue::RuntimeValue => return None,
    };
    Some(ResolvedTypeFactOperand {
        operand: resolved_operand,
        is_subject: !is_typeof_expr(operand, state)
            && matches!(resolved_operand, TypeFactOperand::Param(_))
            && state.is_intrinsic_type(types::expr_type(operand, state), IntrinsicType::Typeref),
    })
}

fn is_typeof_expr(expr: &Expr, state: &State<'_>) -> bool {
    let Expr::Call(call) = unwrap_parentheses(expr) else {
        return false;
    };
    queries::calls::is_intrinsic(call, IntrinsicFn::Typeof, state)
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
