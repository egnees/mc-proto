use std::collections::HashSet;

use crate::search::{
    control::{ApplyFn, ApplyFunctor, GoalFn, InvariantFn, PruneFn},
    error::SearchError,
    searcher::Searcher,
    state::StateTrace,
    step::StateTraceStep,
};

use crate::simulation::system::HashType;

use super::search::ApplyFnWrapper;

////////////////////////////////////////////////////////////////////////////////

pub struct ModelChecker {
    states: Vec<StateTrace>,
    visited: HashSet<HashType>,
}

impl ModelChecker {
    pub fn visited(&self) -> &HashSet<HashType> {
        &self.visited
    }

    pub fn new_with_build(build: impl ApplyFn) -> Self {
        let mut start = StateTrace::new();
        let apply_fn = ApplyFnWrapper::new(build);
        let step = StateTraceStep::Apply(Box::new(apply_fn));
        start.add_step(step);
        Self {
            states: vec![start],
            visited: HashSet::default(),
        }
    }

    pub fn check(
        mut self,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
        mut searcher: impl Searcher,
    ) -> Result<usize, SearchError> {
        searcher.check(
            self.states.clone(),
            &mut self.visited,
            invariant,
            prune,
            goal,
        )
    }

    pub fn collect(
        &mut self,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
        mut searcher: impl Searcher,
    ) -> Result<usize, SearchError> {
        let mut states = Vec::default();
        std::mem::swap(&mut states, &mut self.states);
        self.states = searcher.collect(states, &mut self.visited, invariant, prune, goal)?;

        Ok(self.states.len())
    }

    pub fn apply(&mut self, f: impl ApplyFn) {
        let f = Box::new(ApplyFnWrapper::new(f));
        self.states
            .iter_mut()
            .for_each(|s| s.add_step(StateTraceStep::Apply(f.clone())));
    }
}
