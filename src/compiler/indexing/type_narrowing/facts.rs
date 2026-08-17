use crate::compiler::indexing::IndexState;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::state::TypeFacts;
use crate::utils::parsing::span::Span;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) enum TypeFactOperand<'item> {
    Concrete(&'item StructDefinition),
    Param(&'item Param),
}

pub(super) struct TypeFact<'item> {
    pub(super) operands: [TypeFactOperand<'item>; 2],
    pub(super) condition_node_id: u64,
    pub(super) subject_spans: [Option<Span>; 2],
}

impl<'item> TypeFact<'item> {
    pub(super) fn add(self, state: &mut IndexState<'_, 'item>) {
        let is_new_contradiction = self.add_fact_in_state(state);
        self.record_new_contradiction(is_new_contradiction, state);
    }

    // TODO: review weird formatting
    fn add_fact_in_state(&self, state: &mut IndexState<'_, 'item>) -> bool {
        match self.operands {
            [
                TypeFactOperand::Concrete(left),
                TypeFactOperand::Concrete(right),
            ] => left.id != right.id,
            [
                TypeFactOperand::Param(param),
                TypeFactOperand::Concrete(type_),
            ]
            | [
                TypeFactOperand::Concrete(type_),
                TypeFactOperand::Param(param),
            ] => add_required_type(param, type_, state),
            [TypeFactOperand::Param(left), TypeFactOperand::Param(right)] => {
                merge_params([left, right], state)
            }
        }
    }

    fn record_new_contradiction(
        &self,
        is_new_contradiction: bool,
        state: &mut IndexState<'_, 'item>,
    ) {
        let subject_spans =
            is_new_contradiction.then(|| self.subject_spans.into_iter().flatten().collect());
        state
            .inner
            .set_contradicted_type_fact_subject_spans(self.condition_node_id, subject_spans);
    }
}

// TODO: all the following functions can be associated methods of TypeFact
fn add_required_type<'item>(
    param: &'item Param,
    type_: &'item StructDefinition,
    state: &mut IndexState<'_, 'item>,
) -> bool {
    let previous_facts = type_facts(param, state);
    let mut facts = previous_facts.as_ref().clone();
    facts.add_required_type(type_);
    let is_new_contradiction = !previous_facts.is_contradicted() && facts.is_contradicted();
    merge_type_fact_groups(&[previous_facts], Rc::new(facts), &[param], state);
    is_new_contradiction
}

fn merge_params<'item>(params: [&'item Param; 2], state: &mut IndexState<'_, 'item>) -> bool {
    let previous_facts = params.map(|param| type_facts(param, state));
    let mut facts = previous_facts[0].as_ref().clone();
    facts.add_required_types(&previous_facts[1]);
    let was_contradicted = previous_facts.iter().any(|facts| facts.is_contradicted());
    let is_new_contradiction = !was_contradicted && facts.is_contradicted();
    merge_type_fact_groups(&previous_facts, Rc::new(facts), &params, state);
    is_new_contradiction
}

// TODO: move this function as associated method of TypeNarrowingState
fn type_facts<'item>(param: &Param, state: &IndexState<'_, 'item>) -> Rc<TypeFacts<'item>> {
    state
        .type_narrowing
        .type_facts
        .get(&param.id)
        .cloned()
        .unwrap_or_default()
}

fn merge_type_fact_groups<'item>(
    previous_facts: &[Rc<TypeFacts<'item>>],
    facts: Rc<TypeFacts<'item>>,
    params: &[&Param],
    state: &mut IndexState<'_, 'item>,
) {
    for other_facts in state.type_narrowing.type_facts.values_mut() {
        if previous_facts
            .iter()
            .any(|previous_facts| Rc::ptr_eq(other_facts, previous_facts))
        {
            *other_facts = facts.clone();
        }
    }
    for param in params {
        state
            .type_narrowing
            .type_facts
            .insert(param.id, facts.clone());
    }
}
