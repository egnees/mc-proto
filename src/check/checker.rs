use std::collections::HashSet;

use generic_clone::{store::Store, view::View};

use crate::{
    search::{
        control::{ApplyFn, ApplyFunctor, GoalFn, InvariantFn, PruneFn},
        log::SearchLog,
        searcher::Searcher,
        state::SearchState,
    },
    SearchError,
};

use super::search::ApplyFnWrapper;

////////////////////////////////////////////////////////////////////////////////

pub struct ModelCheckerConfig {
    pub max_states: usize,
    pub max_state_size: usize,
}

impl Default for ModelCheckerConfig {
    fn default() -> Self {
        Self {
            max_states: 50000,
            max_state_size: 1000000,
        }
    }
}

pub struct ModelChecker {
    _state_store: Store,
    states: Vec<View<SearchState>>,
}

impl ModelChecker {
    pub fn new_with_cfg(build: impl ApplyFn, cfg: &ModelCheckerConfig) -> Self {
        let store = Store::new(cfg.max_state_size, cfg.max_states).unwrap();
        let mut view = store.allocate::<SearchState>().unwrap();
        view.enter(|v| build(v.system.handle()));
        let states = vec![view; 1];
        Self {
            _state_store: store,
            states,
        }
    }

    pub fn new_with_build(build: impl ApplyFn) -> Self {
        Self::new_with_cfg(build, &ModelCheckerConfig::default())
    }

    pub fn check(
        mut self,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
        mut searcher: impl Searcher,
    ) -> Result<SearchLog, SearchError> {
        let mut visited = HashSet::default();
        let mut states = Vec::new();
        std::mem::swap(&mut self.states, &mut states);
        searcher.check(states, &mut visited, invariant, prune, goal)
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
            .for_each(|s| s.enter(|s| f.apply(s.system.handle())));
    }

    pub fn for_each(&mut self, f: impl ApplyFn) {
        self.states.iter_mut().for_each(|s| {
            s.enter(|s| f(s.system.handle()));
        });
    }

    pub fn states_count(&self) -> usize {
        self.states.len()
    }
}
