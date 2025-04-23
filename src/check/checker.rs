use std::collections::HashSet;

use crate::{
    search::{
        control::{ApplyFn, ApplyFunctor, GoalFn, InvariantFn, PruneFn},
        log::SearchLog,
        searcher::Searcher,
        state::{SearchState, StateTrace},
        step::StateTraceStep,
    },
    SearchError,
};

use super::search::ApplyFnWrapper;

////////////////////////////////////////////////////////////////////////////////

pub struct ModelChecker {
    states: Vec<StateTrace>,
}

impl ModelChecker {
    pub fn new_with_build(build: impl ApplyFn) -> Self {
        let mut start = StateTrace::new();
        let apply_fn = ApplyFnWrapper::new(build);
        let step = StateTraceStep::Apply(Box::new(apply_fn));
        start.add_step(step);
        Self {
            states: vec![start],
        }
    }

    pub fn check(
        self,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
        mut searcher: impl Searcher,
    ) -> Result<SearchLog, SearchError> {
        let mut visited = HashSet::default();
        searcher.check(self.states.clone(), &mut visited, invariant, prune, goal)
    }

    pub fn collect(
        &mut self,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
        mut searcher: impl Searcher,
    ) -> Result<SearchLog, SearchError> {
        let mut states = Vec::default();
        std::mem::swap(&mut states, &mut self.states);
        let mut visited = HashSet::default();
        let collect_info = searcher.collect(states, &mut visited, invariant, prune, goal)?;
        self.states = collect_info.states;
        Ok(collect_info.log)
    }

    pub fn apply(&mut self, f: impl ApplyFn) {
        let f = Box::new(ApplyFnWrapper::new(f));
        self.states
            .iter_mut()
            .for_each(|s| s.add_step(StateTraceStep::Apply(f.clone())));
    }

    pub fn for_each(&self, f: impl ApplyFn) {
        self.states.iter().for_each(|s| {
            let state = SearchState::from_trace(s).unwrap();
            f(state.system.handle());
        });
    }

    pub fn states_count(&self) -> usize {
        self.states.len()
    }
}
