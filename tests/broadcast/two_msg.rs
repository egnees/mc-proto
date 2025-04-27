use crate::broadcast::{
    check::{check_casual_order, check_someone_deliver},
    common::read_locals,
};

use super::{
    check::check_validity_and_agreement,
    common::{send_local, BuildFn},
};

////////////////////////////////////////////////////////////////////////////////

pub fn collect_until_no_events(
    checker: &mut mc::ModelChecker,
) -> Result<mc::SearchLog, mc::SearchError> {
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);
    checker.collect(
        |_| Ok(()),
        |_| false,
        |s: mc::StateView| {
            if s.system().pending_events() == 0 {
                Ok(())
            } else {
                Err("Contains pending events".into())
            }
        },
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn same_node_no_drop_no_fault_check_causal(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    // send local message
    checker.apply(|s| {
        send_local(s, 0, "m1");
    });

    // step until some node deliver message
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let collect_log = checker.collect(
            |_| Ok(()),
            |_| false,
            move |s| {
                check_someone_deliver(s.system(), nodes)?;
                Ok(())
            },
            s,
        )?;
        println!("collected states={}", checker.states_count());
        println!("log:");
        println!("{}", collect_log);
    }

    // send one more local on the node already delivered
    checker.apply(move |s| {
        let deliver = check_someone_deliver(s.clone(), nodes).unwrap();
        send_local(s, deliver, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(
            move |s| check_casual_order(s.system(), nodes),
            |_| false,
            goal,
            s,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn concurrent_no_drop_no_fault_check_causal(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    // send local message
    checker.apply(|s| {
        send_local(s.clone(), 0, "m1");
        send_local(s, 1, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(
            move |s| check_casual_order(s.system(), nodes),
            |_| false,
            goal,
            s,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_after_recv_no_drop_no_fault_check_causal(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    let first_local = 0;

    // send local message
    checker.apply(move |s| {
        send_local(s, first_local, "m1");
    });

    let wait_on = 1;
    // step until `wait_on` node deliver message
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let collect_log = checker.collect(
            |_| Ok(()),
            |_| false,
            move |s| {
                read_locals(s.system(), wait_on)
                    .map(|v| !v.is_empty())
                    .and_then(|r| if r { Ok(()) } else { Err("No locals".into()) })
            },
            s,
        )?;
        println!("collected states={}", checker.states_count());
        println!("log:");
        println!("{}", collect_log);
    }

    // send one more local on the node already delivered
    checker.apply(move |s| {
        send_local(s, wait_on, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(
            move |s| check_casual_order(s.system(), nodes),
            |_| false,
            goal,
            s,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_after_recv_no_drop_no_fault_check_all(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    let first_local = 0;

    // send local message
    checker.apply(move |s| {
        send_local(s, first_local, "m1");
    });

    let wait_on = 1;
    // step until `wait_on` node deliver message
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let collect_log = checker.collect(
            |_| Ok(()),
            |_| false,
            move |s| {
                read_locals(s.system(), wait_on)
                    .map(|v| !v.is_empty())
                    .and_then(|r| if r { Ok(()) } else { Err("No locals".into()) })
            },
            s,
        )?;
        println!("collected states={}", checker.states_count());
        println!("log:");
        println!("{}", collect_log);
    }

    // send one more local on the node already delivered
    checker.apply(move |s| {
        send_local(s, wait_on, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(
            move |s| check_casual_order(s.system(), nodes),
            |_| false,
            goal,
            s,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_after_recv_no_drop_with_fault_check_all(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    let first_local = 0;

    // send local message
    checker.apply(move |s| {
        send_local(s, first_local, "m1");
    });

    let wait_on = 1;
    // step until `wait_on` node deliver message
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let collect_log = checker.collect(
            |_| Ok(()),
            |_| false,
            move |s| {
                read_locals(s.system(), wait_on)
                    .map(|v| !v.is_empty())
                    .and_then(|r| if r { Ok(()) } else { Err("No locals".into()) })
            },
            s,
        )?;
        println!("collected states={}", checker.states_count());
        println!("log:");
        println!("{}", collect_log);
    }

    // send one more local on the node already delivered
    checker.apply(move |s| {
        send_local(s, wait_on, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::with_node_faults_only(1);
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(
            move |s| check_casual_order(s.system(), nodes),
            |_| false,
            goal,
            s,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn concurrent_with_faults_check_validity_and_agreement(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    // send local message
    checker.apply(|s| {
        send_local(s.clone(), 0, "m1");
        send_local(s, 1, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::with_node_faults_only(1);
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(|_| Ok(()), |_| false, goal, s)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn concurrent_without_faults_check_validity_and_agreement(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));

    // initial
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    // send local message
    checker.apply(|s| {
        send_local(s.clone(), 0, "m1");
        send_local(s, 1, "m2");
    });

    // step until no pending events and all messages are delivered
    {
        let cfg = mc::SearchConfig::no_faults_no_drops();
        let s = mc::BfsSearcher::new(cfg);
        let goal = move |s: mc::StateView| check_validity_and_agreement(s.system(), nodes);
        checker.check(|_| Ok(()), |_| false, goal, s)
    }
}
