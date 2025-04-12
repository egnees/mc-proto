use std::collections::HashSet;

use crate::search::{
    control::{ApplyFn, ApplyFunctor, BuildFn, GoalFn, InvariantFn, PruneFn},
    error::SearchError,
    searcher::Searcher,
    step::StateTraceStep,
    trace::StateTrace,
};

use crate::system::sys::HashType;

use super::search::{
    ApplyFnWrapper, BuildFnWrapper, GoalFnWrapper, InvariantFnWrapper, PruneFnWrapper,
};

////////////////////////////////////////////////////////////////////////////////

pub struct Checker {
    states: Vec<StateTrace>,
    visited: HashSet<HashType>,
}

impl Checker {
    pub fn visited(&self) -> &HashSet<HashType> {
        &self.visited
    }

    pub fn new(build: impl BuildFn) -> Self {
        let build = Box::new(BuildFnWrapper::new(build));
        let start = StateTrace::new(build);
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
        let invariant = InvariantFnWrapper::new(invariant);
        let prune = PruneFnWrapper::new(prune);
        let goal = GoalFnWrapper::new(goal);
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
        let invariant = InvariantFnWrapper::new(invariant);
        let prune = PruneFnWrapper::new(prune);
        let goal = GoalFnWrapper::new(goal);

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
