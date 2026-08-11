use crate::compiler::indexing::IndexState;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::parsing::items::types::StructDefinition;
use crate::compiler::state::TypeFacts;
use crate::compiler::types;
use crate::utils::parsing::span::Span;
use std::rc::Rc;

pub(super) enum TypeFact<'item> {
    Concrete(ConcreteTypeFact<'item>),
    Relation(RelationTypeFact<'item>),
}

impl<'item> TypeFact<'item> {
    pub(super) fn add(self, state: &mut IndexState<'_, 'item>) {
        match self {
            Self::Concrete(fact) => fact.add(state),
            Self::Relation(fact) => fact.add(state),
        }
    }
}

pub(super) struct ConcreteTypeFact<'item> {
    pub(super) param: &'item Param,
    pub(super) type_: &'item StructDefinition,
    pub(super) condition_node_id: u64,
    pub(super) subject_span: Span,
}

impl<'item> ConcreteTypeFact<'item> {
    fn add(self, state: &mut IndexState<'_, 'item>) {
        let previous_facts = type_facts(self.param, state);
        let facts = self.merge(&previous_facts);
        self.record_new_contradiction(&previous_facts, &facts, state);
        merge_type_fact_groups(&[previous_facts], Rc::new(facts), &[self.param], state);
    }

    fn merge(&self, previous_facts: &TypeFacts<'item>) -> TypeFacts<'item> {
        let mut facts = previous_facts.clone();
        facts.add_required_type(self.type_);
        facts
    }

    fn record_new_contradiction(
        &self,
        previous_facts: &TypeFacts<'item>,
        facts: &TypeFacts<'item>,
        state: &mut IndexState<'_, 'item>,
    ) {
        let is_newly_contracted = self.is_new_contradiction(previous_facts, facts, state);
        let subject_spans = is_newly_contracted.then(|| vec![self.subject_span]);
        state
            .inner
            .set_contradicted_type_fact_subject_spans(self.condition_node_id, subject_spans);
    }

    fn is_new_contradiction(
        &self,
        previous_facts: &TypeFacts<'item>,
        facts: &TypeFacts<'item>,
        state: &IndexState<'_, 'item>,
    ) -> bool {
        let param_type = types::param_type(self.param, state.inner);
        let was_contradicted = previous_facts.is_type_contradicted(param_type);
        let is_contradicted = facts.is_type_contradicted(param_type);
        !was_contradicted && is_contradicted
    }
}

pub(super) struct RelationTypeFact<'item> {
    pub(super) params: [&'item Param; 2],
    pub(super) condition_node_id: u64,
    pub(super) subject_spans: [Span; 2],
}

impl<'item> RelationTypeFact<'item> {
    fn add(self, state: &mut IndexState<'_, 'item>) {
        let previous_facts = self.params.map(|param| type_facts(param, state));
        let facts = Self::merge(&previous_facts); // no-fn-check (name shared with concrete facts)
        self.record_new_contradiction(&previous_facts, &facts, state); // no-fn-check (name shared with concrete facts)
        merge_type_fact_groups(&previous_facts, Rc::new(facts), &self.params, state);
    }

    fn merge(previous_facts: &[Rc<TypeFacts<'item>>; 2]) -> TypeFacts<'item> {
        let mut facts = previous_facts[0].as_ref().clone();
        facts.add_required_types(&previous_facts[1]);
        facts
    }

    fn record_new_contradiction(
        &self,
        previous_facts: &[Rc<TypeFacts<'item>>; 2],
        facts: &TypeFacts<'item>,
        state: &mut IndexState<'_, 'item>,
    ) {
        let is_new_contradiction = self.is_new_contradiction(previous_facts, facts, state);
        let subject_spans = is_new_contradiction.then(|| self.subject_spans.into_iter().collect());
        state
            .inner
            .set_contradicted_type_fact_subject_spans(self.condition_node_id, subject_spans);
    }

    fn is_new_contradiction(
        &self,
        previous_facts: &[Rc<TypeFacts<'item>>; 2],
        facts: &TypeFacts<'item>,
        state: &IndexState<'_, 'item>,
    ) -> bool {
        let mut was_contradicted = false;
        let mut is_contradicted = false;
        for (&param, previous_facts) in self.params.iter().zip(previous_facts) {
            let param_type = types::param_type(param, state.inner);
            was_contradicted |= previous_facts.is_type_contradicted(param_type);
            is_contradicted |= facts.is_type_contradicted(param_type);
        }
        !was_contradicted && is_contradicted
    }
}

fn type_facts<'item>(param: &'item Param, state: &IndexState<'_, 'item>) -> Rc<TypeFacts<'item>> {
    state
        .type_narrowing
        .type_facts
        .get(&param.id)
        .cloned()
        .unwrap_or_else(|| {
            let mut facts = TypeFacts::default();
            if let Some(type_) = types::param_type(param, state.inner).struct_ref() {
                facts.add_required_type(type_);
            }
            Rc::new(facts)
        })
}

fn merge_type_fact_groups<'item>(
    previous_facts: &[Rc<TypeFacts<'item>>],
    facts: Rc<TypeFacts<'item>>,
    params: &[&'item Param],
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
