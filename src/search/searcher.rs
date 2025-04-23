use std::collections::HashSet;

use crate::sim::system::HashType;

use super::{
    control::{GoalFn, InvariantFn, PruneFn},
    error::SearchError,
    log::SearchLog,
    state::StateTrace,
};

////////////////////////////////////////////////////////////////////////////////

pub struct CollectInfo {
    pub states: Vec<StateTrace>,
    pub log: SearchLog,
}

////////////////////////////////////////////////////////////////////////////////

pub trait Searcher {
    fn check(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<SearchLog, SearchError>;

    fn collect(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<CollectInfo, SearchError>;
}
