mod facts;

use self::facts::{ConcreteTypeFact, RelationTypeFact, TypeFact};
use crate::compiler::consts::{self, ConstValue};
use crate::compiler::indexing::{IndexState, exprs};
use crate::compiler::item_ref::ItemRef;
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::{BinaryIntrinsicFn, IntrinsicFn};
use crate::compiler::parsing::items::params::Param;
use crate::compiler::queries;
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
    // TODO: find better name
    // TODO: to be seen if really needed
    direct_dependencies: HashMap<u64, Vec<&'item Param>>,
}

impl<'item> TypeNarrowingState<'item> {
    pub(super) fn reset_function(&mut self) {
        self.type_facts.clear();
        self.direct_dependencies.clear();
    }

    pub(super) fn register_direct_dependency(&mut self, param: &'item Param, state: &State<'item>) {
        // TODO: replace by unique if statement
        let ConstValue::Param(type_param) = consts::expr_value(&param.type_, state) else {
            return;
        };
        let Expr::Ident(type_ident) = unwrap_parentheses(&param.type_) else {
            return;
        };
        // TODO: just understand below if condition
        if state.sources.get(&type_ident.id) != Some(&ItemRef::Param(type_param)) {
            return;
        }
        if !is_const_typeref_param(type_param, state) {
            return;
        }
        self.direct_dependencies
            .entry(type_param.id)
            .or_default()
            .push(param);
        // TODO: does it make sense to retrieve existing facts for the param considering is has just been defined?
        let facts = self
            .type_facts
            .get(&type_param.id)
            .cloned()
            .unwrap_or_default();
        // TODO: to be seen if the 2 entries added in type_facts are needed in practice
        self.type_facts.insert(type_param.id, facts.clone());
        self.type_facts.insert(param.id, facts);
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
        && !is_const_typeref_param(param, state.inner) // TODO: just understand why this is necessary
        && let Some(facts) = state.type_narrowing.type_facts.get(&param.id).cloned()
    {
        state.set_expr_type_facts(ident.id, facts);
    }
}

// TODO: move in queries module
// TODO: use is_intrinsic_type is possible
pub(super) fn is_const_typeref_param<'item>(param: &'item Param, state: &State<'item>) -> bool {
    param.const_mark_span().is_some()
        && types::param_type(param, state) == Type::Struct(state.search_prelude_type("typeref"))
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
        narrowing_param(left_arg, state),
        narrowing_param(right_arg, state),
    ) {
        (Some(left_param), Some(right_param)) => Some(TypeFact::Relation(RelationTypeFact {
            params: [left_param, right_param],
            condition_node_id: call.id,
            subject_spans: [left_arg.span(), right_arg.span()],
        })),
        (Some(param), None) => concrete_type_fact(call.id, param, left_arg, right_arg, state),
        (None, Some(param)) => concrete_type_fact(call.id, param, right_arg, left_arg, state),
        (None, None) => None,
    }
}

fn concrete_type_fact<'item>(
    condition_node_id: u64,
    param: &'item Param,
    typeof_expr: &Expr, // TODO: should this param name be generalized too?
    type_expr: &Expr,
    state: &State<'item>,
) -> Option<TypeFact<'item>> {
    if let ConstValue::TypeRef(type_) = consts::expr_value(type_expr, state) {
        Some(TypeFact::Concrete(ConcreteTypeFact {
            param,
            type_,
            condition_node_id,
            subject_span: typeof_expr.span(),
        }))
    } else {
        None
    }
}

fn narrowing_param<'item>(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
    if let Some(param) = typeof_param(expr, state) {
        return Some(param);
    }
    // TODO: replace by unique if statement
    let Expr::Ident(ident) = unwrap_parentheses(expr) else {
        return None;
    };
    let Some(ItemRef::Param(param)) = state.sources.get(&ident.id).copied() else {
        return None;
    };
    is_const_typeref_param(param, state).then_some(param)
}

fn typeof_param<'item>(expr: &Expr, state: &State<'item>) -> Option<&'item Param> {
    // TODO: replace by unique if statement
    let Expr::Call(typeof_call) = unwrap_parentheses(expr) else {
        return None;
    };
    if !queries::calls::is_intrinsic(typeof_call, IntrinsicFn::Typeof, state) {
        return None;
    }
    let Expr::Ident(ident) = unwrap_parentheses(&typeof_call.args[0].value) else {
        return None;
    };
    match state.sources.get(&ident.id).copied() {
        Some(ItemRef::Param(param)) => Some(param),
        Some(ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Fn(_)) | None => {
            None
        }
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
