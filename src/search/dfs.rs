use std::collections::HashSet;

use super::{
    config::SearchConfig,
    control::{GoalFn, InvariantFn, PruneFn},
    error::{InvariantViolation, LivenessViolation, SearchError, SearchErrorKind},
    log::SearchLog,
    searcher::{CollectInfo, Searcher},
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
    ) -> Result<SearchLog, SearchError> {
        let mut log = SearchLog::new();

        let mut goal_achieved = false;

        while let Some(v) = start.pop() {
            log.visited_total += 1;
            let state = SearchState::from_trace(&v).map_err(|kind| SearchError::new(kind, &log))?;
            let system = state.system.handle();
            let h = system.hash();
            if !visited.insert(h) {
                continue;
            }
            log.visited_unique += 1;

            // check invariant
            invariant(system.clone()).map_err(|report| {
                let err = InvariantViolation {
                    trace: v.clone(),
                    log: system.log(),
                    report,
                };
                let kind = SearchErrorKind::InvariantViolation(err);
                SearchError::new(kind, &log)
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
                let err = SearchErrorKind::LivenessViolation(err);
                let err = SearchError::new(err, &log);
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
            Ok(log)
        } else {
            let err = LivenessViolation::no_one();
            let err = SearchErrorKind::LivenessViolation(err);
            let err = SearchError::new(err, &log);
            Err(err)
        }
    }

    fn collect(
        &mut self,
        mut start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<CollectInfo, SearchError> {
        let mut log = SearchLog::new();
        let mut collected = Vec::new();

        while let Some(v) = start.pop() {
            log.visited_total += 1;
            let state = SearchState::from_trace(&v).map_err(|k| SearchError::new(k, &log))?;
            let system = state.system.handle();
            let h = system.hash();
            if !visited.insert(h) {
                continue;
            }
            log.visited_unique += 1;

            // check invariant
            invariant(system.clone()).map_err(|report| {
                let kind = SearchErrorKind::InvariantViolation(InvariantViolation {
                    trace: v.clone(),
                    log: system.log(),
                    report,
                });
                SearchError::new(kind, &log)
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

        Ok(CollectInfo {
            states: collected,
            log,
        })
    }
}
