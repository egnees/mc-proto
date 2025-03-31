use std::collections::HashSet;

use crate::system::sys::HashType;

use super::{
    control::{GoalChecker, InvariantChecker, Pruner},
    error::SearchError,
    trace::Trace,
};

////////////////////////////////////////////////////////////////////////////////

pub trait Searcher {
    fn check(
        &mut self,
        start: Vec<Trace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<usize, SearchError>;

    fn collect(
        &mut self,
        start: Vec<Trace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<Vec<Trace>, SearchError>;
}
