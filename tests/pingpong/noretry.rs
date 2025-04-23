////////////////////////////////////////////////////////////////////////////////

pub struct Ping {
    other: mc::Address,
}

impl Ping {
    pub fn new(other: mc::Address) -> Self {
        Self { other }
    }
}

impl mc::Process for Ping {
    fn on_message(&mut self, from: mc::Address, content: String) {
        assert_eq!(from, self.other);
        mc::send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        mc::send_message(&self.other, content);
    }

    fn hash(&self) -> mc::HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Pong {}

impl Pong {
    pub fn new() -> Self {
        Default::default()
    }
}

impl mc::Process for Pong {
    fn on_message(&mut self, from: mc::Address, content: String) {
        mc::send_message(&from, content.clone());
        mc::send_local(content);
    }

    fn on_local_message(&mut self, _content: String) {
        unreachable!()
    }

    fn hash(&self) -> mc::HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

    use std::{cell::RefCell, rc::Rc, time::Duration};

    use crate::pingpong::common::{make_build, make_goal, make_invariant};

    use super::*;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn reliable_net() {
        let locals = 2;
        let max_drops = 0;

        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = make_goal(locals);
        let build = make_build(
            Duration::from_millis(100),
            Duration::from_millis(600),
            || Rc::new(RefCell::new(Ping::new(mc::Address::new("n2", "pong")))),
            || Rc::new(RefCell::new(Pong::new())),
            locals,
        );

        let cfg = mc::SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .build();
        let searcher = mc::BfsSearcher::new(cfg);
        let checker = mc::ModelChecker::new_with_build(build);
        let checked = checker.check(invariant, prune, goal, searcher).unwrap();

        assert!(checked.visited_unique > 0);
        println!("checked = '{checked}'");
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn unreliable_net() {
        let locals = 2;
        let max_drops = 2;

        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = make_goal(locals);
        let build = make_build(
            Duration::from_millis(100),
            Duration::from_millis(600),
            || Rc::new(RefCell::new(Ping::new(mc::Address::new("n2", "pong")))),
            || Rc::new(RefCell::new(Pong::new())),
            locals,
        );

        let cfg = mc::SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .build();
        let searcher = mc::BfsSearcher::new(cfg);
        let checker = mc::ModelChecker::new_with_build(build);
        let check_result = checker.check(invariant, prune, goal, searcher);
        assert!(check_result.is_err());
        println!("{}", check_result.unwrap_err());
    }
}
