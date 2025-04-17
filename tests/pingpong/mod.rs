mod common;
mod correct;
mod noretry;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, time::Duration};

    use crate::pingpong::{
        common::{make_build, make_goal, make_invariant},
        correct,
    };

    use super::noretry;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn correct_noretry_are_same_on_reliable_net() {
        let locals = 3;
        let max_drops = 0;

        let invariant = make_invariant(locals);
        let prune = |_| false;
        let goal = make_goal(locals);

        let cfg = mc::SearchConfigBuilder::no_faults()
            .max_msg_drops(max_drops)
            .build();
        let checked_correct = {
            let build = make_build(
                Duration::from_millis(100),
                Duration::from_millis(600),
                || {
                    Rc::new(RefCell::new(correct::Ping::new(
                        mc::Address::new("n2", "pong"),
                        Duration::from_secs(2),
                    )))
                },
                || Rc::new(RefCell::new(correct::Pong::new())),
                locals,
            );

            let searcher = mc::BfsSearcher::new(cfg.clone());
            let checker = mc::ModelChecker::new_with_build(build);
            let checked = checker
                .check(invariant.clone(), prune.clone(), goal.clone(), searcher)
                .unwrap();
            checked
        };

        let checked_noretry = {
            let build = make_build(
                Duration::from_millis(100),
                Duration::from_millis(600),
                || {
                    Rc::new(RefCell::new(noretry::Ping::new(mc::Address::new(
                        "n2", "pong",
                    ))))
                },
                || Rc::new(RefCell::new(noretry::Pong::new())),
                locals,
            );

            let searcher = mc::BfsSearcher::new(cfg.clone());
            let checker = mc::ModelChecker::new_with_build(build);
            let checked = checker.check(invariant, prune, goal, searcher).unwrap();
            checked
        };

        assert_eq!(checked_correct, checked_noretry);
    }

    ////////////////////////////////////////////////////////////////////////////////
}
