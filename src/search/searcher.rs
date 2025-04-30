use std::collections::HashSet;

use generic_clone::view::View;

use crate::sim::system::HashType;

use super::{
    control::{GoalFn, InvariantFn, PruneFn},
    error::SearchError,
    log::SearchLog,
    state::SearchState,
};

////////////////////////////////////////////////////////////////////////////////

pub struct CollectInfo {
    pub states: Vec<View<SearchState>>,
    pub log: SearchLog,
}

////////////////////////////////////////////////////////////////////////////////

pub trait Searcher {
    fn check(
        &mut self,
        start: Vec<View<SearchState>>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<SearchLog, SearchError>;

    fn collect(
        &mut self,
        start: Vec<View<SearchState>>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<CollectInfo, SearchError>;
}
