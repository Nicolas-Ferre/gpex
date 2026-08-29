mod facts;
mod operands;

use self::facts::TypeFact;
use crate::compiler::indexing::type_narrowing::operands::ResolvedTypeFactOperand;
use crate::compiler::indexing::{IndexState, exprs};
use crate::compiler::parsing::exprs::Expr;
use crate::compiler::parsing::exprs::calls::Call;
use crate::compiler::parsing::exprs::idents::Ident;
use crate::compiler::parsing::items::fns::BinaryIntrinsicFn;
use crate::compiler::queries;
use crate::compiler::state::State;
use crate::compiler::state::type_facts::{TypeFactContext, TypeFactSubject, TypeFacts};
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

// TODO: flatten?
#[derive(Default)]
pub(super) struct TypeNarrowingState<'item> {
    type_facts: Rc<TypeFactContext<'item>>,
}

impl<'item> TypeNarrowingState<'item> {
    pub(crate) fn reset(&mut self) {
        self.type_facts = Rc::default();
    }

    fn type_facts(&self, subject: TypeFactSubject<'item>) -> Rc<TypeFacts<'item>> {
        self.type_facts.get(&subject).cloned().unwrap_or_default()
    }
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

pub(super) fn index_ident(ident: &Ident, state: &mut IndexState<'_, '_>) {
    let context = (!state.type_narrowing.type_facts.is_empty())
        .then(|| state.type_narrowing.type_facts.clone());
    state.set_expr_type_fact_context(ident.id, context);
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
    let left_operand = ResolvedTypeFactOperand::resolve(left_arg, state)?;
    let right_operand = ResolvedTypeFactOperand::resolve(right_arg, state)?;
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
