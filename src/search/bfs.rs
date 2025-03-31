use std::collections::{HashSet, VecDeque};

use super::{
    control::{GoalChecker, InvariantChecker, Pruner},
    error::{InvariantViolation, LivenessViolation, SearchError},
    searcher::Searcher,
    trace::Trace,
    SearchConfig,
};

use crate::system::sys::HashType;

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
        start: Vec<Trace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<usize, SearchError> {
        let mut queue: VecDeque<Trace> = start.into_iter().collect();

        let mut cnt = 0;
        let mut goal_achieved = false;
        while let Some(v) = queue.pop_front() {
            cnt += 1;

            let sys = v.system();
            let h = sys.hash();
            if !visited.insert(h) {
                continue;
            }

            // check invariant
            invariant.check(sys.handle()).map_err(|report| {
                let err = InvariantViolation {
                    trace: v.clone(),
                    log: sys.log(),
                    report,
                };
                SearchError::InvariantViolation(err)
            })?;

            // check goal achieved
            if goal.check(sys.handle()) {
                goal_achieved = true;
                continue;
            }

            // check prune
            if prune.check(sys.handle()) {
                continue;
            }

            // error if no transitions available
            let steps = sys.search_steps(&self.cfg);
            if steps.is_empty() {
                let err = LivenessViolation::this_one(v, sys.log());
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
        start: Vec<Trace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<Vec<Trace>, SearchError> {
        let mut collected = Vec::new();

        let mut queue: VecDeque<Trace> = start.into_iter().collect();

        while let Some(v) = queue.pop_front() {
            let sys = v.system();
            let h = sys.hash();
            if !visited.insert(h) {
                continue;
            }

            // check invariant
            invariant.check(sys.handle()).map_err(|report| {
                let err = InvariantViolation {
                    trace: v.clone(),
                    log: sys.log(),
                    report,
                };
                SearchError::InvariantViolation(err)
            })?;

            // check goal achieved
            if goal.check(sys.handle()) {
                collected.push(v);
                continue;
            }

            // check prune
            if prune.check(sys.handle()) {
                continue;
            }

            // check depth restriction
            if v.depth() >= self.cfg.max_depth.unwrap_or(usize::MAX) {
                continue;
            }

            // branch
            let steps = sys.search_steps(&self.cfg);
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
