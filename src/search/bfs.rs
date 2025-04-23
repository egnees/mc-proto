use std::collections::{HashSet, VecDeque};

use super::{
    config::SearchConfig,
    control::{GoalFn, InvariantFn, PruneFn},
    error::{InvariantViolation, LivenessViolation, SearchErrorKind},
    log::SearchLog,
    searcher::{CollectInfo, Searcher},
    state::{SearchState, StateTrace},
};

use crate::{sim::system::HashType, SearchError, StateView};

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
    ) -> Result<SearchLog, SearchError> {
        let mut log = SearchLog::new();

        let mut queue: VecDeque<StateTrace> = start.into_iter().collect();

        let mut goal_achieved = false;
        while let Some(v) = queue.pop_front() {
            log.visited_total += 1;
            let state = SearchState::from_trace(&v).map_err(|k| SearchError::new(k, &log))?;
            let system = state.system.handle();
            let h = system.hash();
            let already_meet = !visited.insert(h);
            if !already_meet {
                log.visited_unique += 1;
            }

            // make state view
            let view = StateView::new(&state, v.clone());

            // check invariant
            invariant(view.clone()).map_err(|report| {
                let err = InvariantViolation {
                    trace: v.clone(),
                    log: system.log(),
                    report,
                };
                let kind = SearchErrorKind::InvariantViolation(err);
                SearchError::new(kind, &log)
            })?;

            // check goal achieved
            if goal(view.clone()) {
                goal_achieved = true;
                continue;
            }

            // check prune
            if prune(view) {
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
            if v.depth() >= self.cfg.max_depth.unwrap_or(usize::MAX) || already_meet {
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
            Ok(log)
        } else {
            let kind = SearchErrorKind::LivenessViolation(LivenessViolation::no_one());
            let err = SearchError::new(kind, &log);
            Err(err)
        }
    }

    fn collect(
        &mut self,
        start: Vec<StateTrace>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<CollectInfo, SearchError> {
        let mut log = SearchLog::new();
        let mut collected = Vec::new();
        let mut queue: VecDeque<StateTrace> = start.into_iter().collect();

        while let Some(v) = queue.pop_front() {
            log.visited_total += 1;
            let state = SearchState::from_trace(&v).map_err(|k| SearchError::new(k, &log))?;
            let system = state.system.handle();
            let h = system.hash();
            let already_meet = !visited.insert(h);
            if !already_meet {
                log.visited_unique += 1;
            }

            // make state view
            let view = StateView::new(&state, v.clone());

            // check invariant
            invariant(view.clone()).map_err(|report| {
                let err = InvariantViolation {
                    trace: v.clone(),
                    log: system.log(),
                    report,
                };
                let kind = SearchErrorKind::InvariantViolation(err);
                SearchError::new(kind, &log)
            })?;

            // check goal achieved
            if goal(view.clone()) {
                collected.push(v);
                continue;
            }

            // check prune
            if prune(view) {
                continue;
            }

            // check depth restriction and already meet condition
            if v.depth() >= self.cfg.max_depth.unwrap_or(usize::MAX) || already_meet {
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

        Ok(CollectInfo {
            states: collected,
            log,
        })
    }
}
