use crate::compiler::indexing::IndexState;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::state::{State, TypeFacts};
use crate::compiler::types::{self, Type};
use crate::utils::parsing::span::Span;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) enum TypeFactOperand<'item> {
    Concrete(&'item StructDefinition),
    Param(&'item Param),
}

impl<'item> TypeFactOperand<'item> {
    pub(super) fn from_param(param: &'item Param, state: &State<'item>) -> Self {
        match types::param_type(param, state) {
            Type::Struct(type_) => Self::Concrete(type_),
            Type::Param(type_param) => Self::Param(type_param),
            Type::Wildcard(_) | Type::NoReturn | Type::Unknown => Self::Param(param),
        }
    }
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

    fn add_fact_in_state(&self, state: &mut IndexState<'_, 'item>) -> bool {
        use TypeFactOperand::{Concrete, Param};
        match self.operands {
            [Concrete(left), Concrete(right)] => left.id != right.id,
            [Param(param), Concrete(type_)] | [Concrete(type_), Param(param)] => {
                Self::add_required_type(param, type_, state)
            }
            [Param(left), Param(right)] => Self::merge_params([left, right], state),
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

    fn add_required_type(
        param: &'item Param,
        type_: &'item StructDefinition,
        state: &mut IndexState<'_, 'item>,
    ) -> bool {
        let previous_facts = state.type_narrowing.type_facts(param);
        let mut facts = previous_facts.as_ref().clone();
        facts.add_required_type(type_);
        let is_new_contradiction = !previous_facts.has_contradiction() && facts.has_contradiction();
        Self::merge_type_fact_groups(&[previous_facts], Rc::new(facts), &[param], state);
        is_new_contradiction
    }

    fn merge_params(params: [&'item Param; 2], state: &mut IndexState<'_, 'item>) -> bool {
        let previous_facts = params.map(|param| state.type_narrowing.type_facts(param));
        let mut facts = previous_facts[0].as_ref().clone();
        facts.add_required_types(&previous_facts[1]);
        let was_contradicted = previous_facts.iter().any(|facts| facts.has_contradiction());
        let is_new_contradiction = !was_contradicted && facts.has_contradiction();
        Self::merge_type_fact_groups(&previous_facts, Rc::new(facts), &params, state);
        is_new_contradiction
    }

    fn merge_type_fact_groups(
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
}
