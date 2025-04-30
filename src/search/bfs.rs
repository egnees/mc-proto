use std::collections::{HashSet, VecDeque};

use generic_clone::{in_global, view::View};

use super::{
    config::SearchConfig,
    control::{GoalFn, InvariantFn, PruneFn},
    error::{AllPruned, InvariantViolation, LivenessViolation, SearchErrorKind},
    log::SearchLog,
    searcher::{CollectInfo, Searcher},
    state::SearchState,
};

use crate::{search::error::Cycled, sim::system::HashType, SearchError};

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
        start: Vec<View<SearchState>>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<SearchLog, SearchError> {
        let mut log = SearchLog::new();

        let mut queue: VecDeque<View<SearchState>> = start.into_iter().collect();

        let mut last_prune = None;
        let mut last_already_meet = None;

        let mut goal_achieved = false;

        let mut steps_storage = Vec::new();
        steps_storage.reserve(32);

        while let Some(mut v) = queue.pop_front() {
            log.visited_total += 1;

            let mut func = |s: &mut SearchState| {
                let view = s.view();
                let system = view.system().clone();

                let h = system.hash();
                let already_meet = in_global(|| !visited.insert(h));
                if !already_meet {
                    log.visited_unique += 1;
                }

                // check invariant
                invariant(view.clone()).map_err(|report| {
                    let err = InvariantViolation {
                        log: view.system().log(),
                        report,
                    };
                    let kind = SearchErrorKind::InvariantViolation(err);
                    in_global(|| SearchError::new(kind.clone(), &log))
                })?;

                // check goal achieved
                let goal_check_result = goal(view.clone());
                if goal_check_result.is_ok() {
                    goal_achieved = true;
                    return Ok(false);
                }

                // check prune
                if prune(view) {
                    last_prune = Some(in_global(|| AllPruned::new(system.log())));
                    return Ok(false);
                }

                // error if no transitions available
                let mut steps = s.gen.borrow().steps(system.clone(), &self.cfg);
                if steps.is_empty() {
                    let err = LivenessViolation::new(system.log(), goal_check_result.unwrap_err());
                    let err = SearchErrorKind::LivenessViolation(err);
                    let err = SearchError::new(in_global(|| err.clone()), &log);
                    return Err(err);
                }

                // check already meet condition
                if already_meet {
                    last_already_meet = Some(in_global(|| Cycled::new(system.log(), h)));
                    return Ok(false);
                }

                in_global(|| steps_storage.append(&mut steps));

                Ok(true)
            };

            let do_branch = v.enter(|s| func(s))?;
            if do_branch {
                // branch
                for step in steps_storage.iter() {
                    let mut u = v.clone();
                    u.enter(|s| {
                        s.apply_step(step)
                            .map_err(|e| SearchError::new(in_global(|| e.clone()), &log))
                    })?;
                    queue.push_back(u);
                }
            }

            steps_storage.clear();
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

    ////////////////////////////////////////////////////////////////////////////////

    fn collect(
        &mut self,
        start: Vec<View<SearchState>>,
        visited: &mut HashSet<HashType>,
        invariant: impl InvariantFn,
        prune: impl PruneFn,
        goal: impl GoalFn,
    ) -> Result<CollectInfo, SearchError> {
        let mut log = SearchLog::new();

        let mut queue: VecDeque<View<SearchState>> = start.into_iter().collect();

        let mut steps_storage = Vec::new();
        steps_storage.reserve(32);

        let mut collected = Vec::new();

        while let Some(mut v) = queue.pop_front() {
            log.visited_total += 1;

            let mut should_collect = false;

            let mut func = |s: &mut SearchState| {
                let view = s.view();
                let system = view.system().clone();

                let h = system.hash();
                let already_meet = in_global(|| !visited.insert(h));
                if !already_meet {
                    log.visited_unique += 1;
                }

                // check invariant
                invariant(view.clone()).map_err(|report| {
                    let err = InvariantViolation {
                        log: view.system().log(),
                        report,
                    };
                    let kind = SearchErrorKind::InvariantViolation(err);
                    in_global(|| SearchError::new(kind.clone(), &log))
                })?;

                // check prune
                if prune(view.clone()) || already_meet {
                    return Ok(false);
                }

                // check goal achieved
                let goal_check_result = goal(view);
                if goal_check_result.is_ok() {
                    should_collect = true;
                    return Ok(false);
                }

                // error if no transitions available
                let mut steps = s.gen.borrow().steps(system.clone(), &self.cfg);
                in_global(|| steps_storage.append(&mut steps));

                Ok(true)
            };

            let do_branch = v.enter(|s| func(s))?;
            if should_collect {
                collected.push(v.clone());
            }
            if do_branch {
                // branch
                for step in steps_storage.iter() {
                    let mut u = v.clone();
                    u.enter(|s| {
                        s.apply_step(step)
                            .map_err(|e| SearchError::new(in_global(|| e.clone()), &log))
                    })?;
                    queue.push_back(u);
                }
            }

            steps_storage.clear();
        }

        Ok(CollectInfo {
            states: collected,
            log,
        })
    }
}
