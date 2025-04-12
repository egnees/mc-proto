use std::collections::HashSet;

use super::{
    control::{GoalChecker, InvariantChecker, Pruner},
    error::{InvariantViolation, LivenessViolation, SearchError},
    searcher::Searcher,
    trace::StateTrace,
    SearchConfig,
};

use crate::system::sys::HashType;

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
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<usize, SearchError> {
        let mut cnt = 0;
        let mut goal_achieved = false;
        while let Some(v) = start.pop() {
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
        invariant: impl InvariantChecker,
        prune: impl Pruner,
        goal: impl GoalChecker,
    ) -> Result<Vec<StateTrace>, SearchError> {
        let mut collected = Vec::new();

        while let Some(v) = start.pop() {
            let sys = v.system();
            let h = sys.hash();
            if !visited.insert(h) {
                continue;
            }

            // check invariant
            invariant.check(sys.handle()).map_err(|report| {
                SearchError::InvariantViolation(InvariantViolation {
                    trace: v.clone(),
                    log: sys.log(),
                    report,
                })
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

            // branch
            sys.search_steps(&self.cfg)
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
