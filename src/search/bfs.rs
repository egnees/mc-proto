use std::collections::{HashSet, VecDeque};

use super::{
    config::SearchConfig,
    control::{GoalFn, InvariantFn, PruneFn},
    error::{InvariantViolation, LivenessViolation, SearchError},
    searcher::Searcher,
    state::{SearchState, StateTrace},
};

use crate::simulation::system::HashType;

////////////////////////////////////////////////////////////////////////////////

pub struct BfsSearcher {
    cfg: SearchConfig,
}

impl BfsSearcher {
    pub fn new(cfg: SearchConfig) -> Self {
        Self { cfg }
    }
}

impl Searcher for BfsSearcher {
    fn check(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<usize, SearchError> {
        let mut queue: VecDeque<StateTrace> = start.into_iter().collect();

        let mut cnt = 0;
        let mut goal_achieved = false;
        while let Some(v) = queue.pop_front() {
            cnt += 1;

            let state = SearchState::from_trace(&v);
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
                .for_each(|u| queue.push_back(u));
        }

        if goal_achieved {
            Ok(cnt)
        } else {
            Err(SearchError::LivenessViolation(LivenessViolation::no_one()))
        }
    }

    fn collect(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<Vec<StateTrace>, SearchError> {
        let mut collected = Vec::new();

        let mut queue: VecDeque<StateTrace> = start.into_iter().collect();

        while let Some(v) = queue.pop_front() {
            let state = SearchState::from_trace(&v);
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
                collected.push(v);
                continue;
            }

            // check prune
            if prune(system.clone()) {
                continue;
            }

            // check depth restriction
            if v.depth() >= self.cfg.max_depth.unwrap_or(usize::MAX) {
                continue;
            }

            // branch
            let steps = state.gen.borrow().steps(system, &self.cfg);
            steps
                .iter()
                .map(|s| {
                    let mut u = v.clone();
                    u.add_step(s.clone());
                    u
                })
                .for_each(|u| queue.push_back(u));
        }

        Ok(collected)
    }
}
