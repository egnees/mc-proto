use std::collections::HashSet;

use crate::mc::search::{
    control::{ApplyFn, ApplyFunctor, GoalFn, InvariantFn, PruneFn},
    error::SearchError,
    log::SearchLog,
    searcher::Searcher,
    state::{SearchState, StateTrace},
    step::StateTraceStep,
};

use super::wrapper::ApplyFnWrapper;

////////////////////////////////////////////////////////////////////////////////

/// Allows to control model checking workflow.
///
/// Operates with search states, which corresponds to the states of the testing system.
/// Initially there is only one search
/// state, produced with build function on MC initiation ([ModelChecker::new_with_build]).
///
/// On each collect iteration (see [ModelChecker::collect]) the search starts from every stored state.
/// All collected states are stored for the future interations. Then, the check method can be called
/// (see [ModelChecker::check]), which initiates search from every stored state and checks provided properties
/// of the system.
pub struct ModelChecker {
    states: Vec<StateTrace>,
}

impl ModelChecker {
    /// Make new checker with method, which initializes system model.
    pub fn new_with_build(build: impl ApplyFn) -> Self {
        let mut start = StateTrace::new();
        let apply_fn = ApplyFnWrapper::new(build);
        let step = StateTraceStep::Apply(Box::new(apply_fn));
        start.add_step(step);
        Self {
            states: vec![start],
        }
    }

    /// Run check with the specified searcher.
    /// Checks provided invaraint for each visited state.
    /// Prunes not relevant states indicated by provided prune predicate.
    /// Checks livenes property using the provided goal predicate.
    ///
    /// This method borrows checker by value,
    /// because it is the terminal method, which ends the search pipeline.
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

    /// Run search using the provided searcher and collect states,
    /// which satisfy provided goal predicate.
    /// Check provided invariant for each state visited during the search.
    /// Prune not relevant states which satisfy provided prune predicate.
    ///
    /// All collected states will be stored in the checked and will be used in
    /// the future pipeline iterations.
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

    /// Apply function for each current stored state.
    /// Allows to crash nodes in some states or send local messages for processes,
    /// for example.
    pub fn apply(&mut self, f: impl ApplyFn) {
        let f = Box::new(ApplyFnWrapper::new(f));
        self.states
            .iter_mut()
            .for_each(|s| s.add_step(StateTraceStep::Apply(f.clone())));
    }

    /// Apply provided fucntion without mutating stored states.
    pub fn for_each(&self, f: impl ApplyFn) {
        self.states.iter().for_each(|s| {
            let state = SearchState::from_trace(s).unwrap();
            f(state.system.handle());
        });
    }

    /// Returns the number of current stored states.
    pub fn states_count(&self) -> usize {
        self.states.len()
    }
}
