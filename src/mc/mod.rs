//! Allows to test system model [`crate::model`] using model checking.

mod checker;
mod search;
mod tracker;
mod wrapper;

////////////////////////////////////////////////////////////////////////////////

pub use search::{
    bfs::BfsSearcher,
    config::{SearchConfig, SearchConfigBuilder},
    control::ApplyFn,
    control::GoalFn,
    control::InvariantFn,
    control::PruneFn,
    dfs::DfsSearcher,
    error,
    log::SearchLog,
    state::StateView,
};

pub use checker::ModelChecker;

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
