use std::collections::HashSet;

use crate::sim::system::HashType;

use super::{
    control::{GoalFn, InvariantFn, PruneFn},
    error::SearchError,
    state::StateTrace,
};

////////////////////////////////////////////////////////////////////////////////

pub trait Searcher {
    fn check(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<usize, SearchError>;

    fn collect(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<Vec<StateTrace>, SearchError>;
}
