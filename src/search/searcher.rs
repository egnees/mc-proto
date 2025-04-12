use std::collections::HashSet;

use crate::system::sys::HashType;

use super::{
    control::{GoalChecker, InvariantChecker, Pruner},
    error::SearchError,
    trace::StateTrace,
};

////////////////////////////////////////////////////////////////////////////////

pub trait Searcher {
    fn check(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<usize, SearchError>;

    fn collect(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<Vec<StateTrace>, SearchError>;
}
