use std::collections::{HashSet, VecDeque};

use super::{
    config::SearchConfig,
    control::{GoalFn, InvariantFn, PruneFn},
    error::{AllPruned, InvariantViolation, LivenessViolation, SearchErrorKind},
    log::SearchLog,
    searcher::{CollectInfo, Searcher},
    state::{SearchState, StateTrace},
};

use crate::{mc::error::Cycled, mc::error::SearchError, mc::StateView, model::system::HashType};

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

        let mut last_prune = None;
        let mut last_already_meet = None;

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
            let goal_check_result = goal(view.clone());
            if goal_check_result.is_ok() {
                goal_achieved = true;
                continue;
            }

            // check prune
            if prune(view) {
                last_prune = Some(AllPruned::new(v.clone(), system.log()));
                continue;
            }

            // error if no transitions available
            let steps = state.gen.borrow().steps(system.clone(), &self.cfg);
            if steps.is_empty() {
                let err = LivenessViolation::new(v, system.log(), goal_check_result.unwrap_err());
                let err = SearchErrorKind::LivenessViolation(err);
                let err = SearchError::new(err, &log);
                return Err(err);
            }

            // check already meet condition
            if already_meet {
                last_already_meet = Some(Cycled::new(v, system.log(), h));
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
        } else if let Some(last_prune) = last_prune {
            let err = SearchErrorKind::AllPruned(last_prune);
            let err = SearchError::new(err, &log);
            Err(err)
        } else {
            let err = SearchErrorKind::Cycled(last_already_meet.unwrap());
            let err = SearchError::new(err, &log);
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

            // check prune and already meet condition
            if prune(view.clone()) || already_meet {
                continue;
            }

            // check goal achieved
            if goal(view).is_ok() {
                collected.push(v);
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
