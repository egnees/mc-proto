use std::collections::HashSet;

use super::{
    config::SearchConfig,
    control::{GoalFn, InvariantFn, PruneFn},
    error::{InvariantViolation, LivenessViolation, SearchError},
    searcher::Searcher,
    state::{SearchState, StateTrace},
};

use crate::sim::system::HashType;

////////////////////////////////////////////////////////////////////////////////

pub struct DfsSearcher {
    cfg: SearchConfig,
}

impl DfsSearcher {
    pub fn new(cfg: SearchConfig) -> Self {
        Self { cfg }
    }
}

impl Searcher for DfsSearcher {
    fn check(
        &mut self,
        mut start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<usize, SearchError> {
        let mut cnt = 0;
        let mut goal_achieved = false;
        while let Some(v) = start.pop() {
            cnt += 1;

            let state = SearchState::from_trace(&v)?;
            let system = state.system.handle();
            let h = system.hash();
            if !visited.insert(h) {
                continue;
            }

            // check invariant
            invariant(system.clone()).map_err(|report| {
                let err = InvariantViolation {
                    trace: v.clone(),
                    log: system.log(),
                    report,
                };
                SearchError::InvariantViolation(err)
            })?;

            // check goal achieved
            if goal(system.clone()) {
                goal_achieved = true;
                continue;
            }

            // check prune
            if prune(system.clone()) {
                continue;
            }

            // error if no transitions available
            let steps = state.gen.borrow().steps(system.clone(), &self.cfg);
            if steps.is_empty() {
                let err = LivenessViolation::this_one(v, system.log());
                let err = SearchError::LivenessViolation(err);
                return Err(err);
            }

            // check depth restriction
            if v.depth() >= self.cfg.max_depth.unwrap_or(usize::MAX) {
                continue;
            }

            // branch
            steps
                .iter()
                .map(|s| {
                    let mut u = v.clone();
                    u.add_step(s.clone());
                    u
                })
                .for_each(|u| start.push(u));
        }

        if goal_achieved {
            Ok(cnt)
        } else {
            let err = LivenessViolation::no_one();
            Err(SearchError::LivenessViolation(err))
        }
    }

    fn collect(
        &mut self,
        mut start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<Vec<StateTrace>, SearchError> {
        let mut collected = Vec::new();

        while let Some(v) = start.pop() {
            let state = SearchState::from_trace(&v)?;
            let system = state.system.handle();
            let h = system.hash();
            if !visited.insert(h) {
                continue;
            }

            // check invariant
            invariant(system.clone()).map_err(|report| {
                SearchError::InvariantViolation(InvariantViolation {
                    trace: v.clone(),
                    log: system.log(),
                    report,
                })
            })?;

            // check goal achieved
            if goal(system.clone()) {
                collected.push(v);
                continue;
            }

            // check prune
            if prune(system.clone()) {
                continue;
            }

            // branch
            let steps = state.gen.borrow().steps(system.clone(), &self.cfg);

            steps
                .iter()
                .map(|s| {
                    let mut u = v.clone();
                    u.add_step(s.clone());
                    u
                })
                .for_each(|u| start.push(u));
        }

        Ok(collected)
    }
}
