use std::{cell::RefCell, collections::BTreeSet, rc::Rc, time::Duration};

////////////////////////////////////////////////////////////////////////////////

pub use crate::prelude::*;
use crate::{model::net::send_message, HashType};

////////////////////////////////////////////////////////////////////////////////

pub struct Ping {
    other: Address,
    duration: Duration,
    state: Rc<RefCell<PingState>>,
}

#[derive(PartialEq, Eq, Default)]
struct PingState {
    waiting: BTreeSet<String>,
}

impl PingState {
    fn hash(&self) -> HashType {
        self.waiting.len() as HashType
    }
}

impl Ping {
    pub fn new(other: Address, duration: Duration) -> Self {
        Self {
            other,
            duration,
            state: Rc::new(RefCell::new(Default::default())),
        }
    }
}

impl Process for Ping {
    fn on_message(&mut self, from: Address, content: String) {
        assert_eq!(from, self.other);
        if self.state.borrow_mut().waiting.remove(&content) {
            send_local(content);
        }
    }

    fn on_local_message(&mut self, content: String) {
        self.state.borrow_mut().waiting.insert(content.clone());
        let duration = self.duration;
        spawn({
            let state = self.state.clone();
            let receiver = self.other.clone();
            async move {
                while state.borrow().waiting.contains(&content) {
                    send_message(&receiver, content.clone());
                    sleep(duration).await;
                }
            }
        });
    }

    fn hash(&self) -> HashType {
        self.state.borrow().hash()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Pong {
    delivered: BTreeSet<String>,
}

impl Pong {
    pub fn new() -> Self {
        Self {
            delivered: Default::default(),
        }
    }
}

impl Process for Pong {
    fn on_message(&mut self, from: Address, content: String) {
        send_message(&from, content.clone());
        if self.delivered.insert(content.clone()) {
            send_local(content);
        }
    }

    fn on_local_message(&mut self, _content: String) {
        unreachable!()
    }

    fn hash(&self) -> HashType {
        self.delivered.len() as HashType
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::mc::{
        self,
        tests::pingpong::common::{make_build, make_goal, make_invariant},
    };

    use super::*;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn correct_is_correct() {
        let locals = 2;
        let max_drops = 1;

        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = make_goal(locals);
        let build = make_build(
            Duration::from_millis(100),
            Duration::from_millis(600),
            || {
                Rc::new(RefCell::new(Ping::new(
                    Address::new("n2", "pong"),
                    Duration::from_secs(1),
                )))
            },
            || Rc::new(RefCell::new(Pong::new())),
            locals,
        );

        let cfg = mc::SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .max_node_faults(0)
            .build();

        let checked_bfs = {
            let searcher = mc::BfsSearcher::new(cfg.clone());
            let checker = mc::ModelChecker::new_with_build(build.clone());
            let checked = checker
                .check(invariant.clone(), prune.clone(), goal.clone(), searcher)
                .unwrap();
            checked
        };

        let checked_dfs = {
            let searcher = mc::DfsSearcher::new(cfg);
            let checker = mc::ModelChecker::new_with_build(build);
            let checked = checker.check(invariant, prune, goal, searcher).unwrap();
            checked
        };

        assert!(checked_bfs.visited_unique > 0);
        assert!(checked_dfs.visited_unique > 0);
        println!("checked_bfs = '{checked_bfs}'");
        println!("checked_dfs = '{checked_dfs}'");
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn correct_is_correct_no_drops() {
        let locals = 2;
        let max_drops = 0;

        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = make_goal(locals);
        let build = make_build(
            Duration::from_millis(100),
            Duration::from_millis(600),
            || {
                Rc::new(RefCell::new(Ping::new(
                    Address::new("n2", "pong"),
                    Duration::from_secs(1),
                )))
            },
            || Rc::new(RefCell::new(Pong::new())),
            locals,
        );

        let cfg = mc::SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .max_node_faults(0)
            .build();
        let searcher = mc::BfsSearcher::new(cfg);
        let checker = mc::ModelChecker::new_with_build(build);
        let checked = checker.check(invariant, prune, goal, searcher).unwrap();

        assert!(checked.visited_unique > 0);
        println!("checked = '{checked}'");
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn on_node_crash() {
        let locals = 2;
        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = make_goal(locals);
        let build = make_build(
            Duration::from_millis(100),
            Duration::from_millis(600),
            || {
                Rc::new(RefCell::new(Ping::new(
                    Address::new("n2", "pong"),
                    Duration::from_secs(1),
                )))
            },
            || Rc::new(RefCell::new(Pong::new())),
            locals,
        );
        let cfg = mc::SearchConfigBuilder::new()
            .max_node_faults(1)
            .max_disk_faults(0)
            .max_msg_drops(0)
            .build();
        let searcher = mc::DfsSearcher::new(cfg);
        let checker = mc::ModelChecker::new_with_build(build);
        let check_result = checker.check(invariant, prune, goal, searcher);
        assert!(check_result.is_err());
        println!("{}", check_result.err().unwrap());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn on_node_crash_manually() {
        let locals = 2;
        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = |s: mc::StateView| {
            let is_not_empty = !s.system().read_locals("n2", "pong").unwrap().is_empty();
            if is_not_empty {
                Ok(())
            } else {
                Err("No locals received from n2:pong".into())
            }
        };
        let build = make_build(
            Duration::from_millis(100),
            Duration::from_millis(600),
            || {
                Rc::new(RefCell::new(Ping::new(
                    Address::new("n2", "pong"),
                    Duration::from_secs(1),
                )))
            },
            || Rc::new(RefCell::new(Pong::new())),
            locals,
        );
        let cfg = mc::SearchConfigBuilder::no_faults()
            .max_msg_drops(0)
            .build();
        let searcher = mc::BfsSearcher::new(cfg.clone());
        let mut checker = mc::ModelChecker::new_with_build(build);
        let log = checker
            .collect(invariant.clone(), prune.clone(), goal, searcher)
            .unwrap();
        assert!(log.visited_unique > 0);
        println!("{}", log);
        println!("collected = {}", checker.states_count());
        checker.apply(|s| s.crash_node("n2").unwrap());
        checker.for_each(|s| println!("{}", s.log()));
        let searcher = mc::BfsSearcher::new(cfg);
        let goal = make_goal(locals);
        let result = checker.check(invariant, prune, goal, searcher);
        assert!(result.is_err());
        println!("{}", result.err().unwrap());
    }
}
