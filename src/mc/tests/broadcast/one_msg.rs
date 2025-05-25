use super::{
    check::{check_locals_cnt, check_someone_deliver, check_validity_and_agreement},
    common::BuildFn,
};

use crate::mc;

////////////////////////////////////////////////////////////////////////////////

pub fn collect_until_no_events(
    checker: &mut mc::ModelChecker,
) -> Result<mc::SearchLog, mc::error::SearchError> {
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

pub fn no_drops(build: impl BuildFn) -> Result<mc::SearchLog, mc::error::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect finished");
    println!("{collect_log}");
    println!("collected: {}", checker.states_count());

    checker.apply(move |s| s.send_local(&"0:bcast".into(), &messages[0]).unwrap());

    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);

    checker.check(
        move |s| check_locals_cnt(s.system(), nodes, 1),
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn node_crash_after_someone_delivery(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::error::SearchError> {
    let nodes = 3;

    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect log:");
    println!("{collect_log}");
    println!("collected: {}", checker.states_count());

    checker.apply(|s| s.send_local(&"0:bcast".into(), "0:Hello").unwrap());

    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);
    let collect_log = checker.collect(
        |_| Ok(()),
        |_| false,
        move |s| {
            check_someone_deliver(s.system(), nodes)?;
            Ok(())
        },
        searcher,
    )?;

    println!("Collect log:");
    println!("{collect_log}");

    let cfg = mc::SearchConfig::with_node_faults_only(1);
    let searcher = mc::BfsSearcher::new(cfg);
    checker.check(
        |_| Ok(()),
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn udp_drops_bfs(build: impl BuildFn) -> Result<mc::SearchLog, mc::error::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect finished");
    println!("{collect_log}");
    println!("collected: {}", checker.states_count());

    checker.apply(move |s| s.send_local(&"0:bcast".into(), &messages[0]).unwrap());

    let cfg = mc::SearchConfig::no_faults_with_drops(2);
    let searcher = mc::BfsSearcher::new(cfg);

    checker.check(
        move |s| check_locals_cnt(s.system(), nodes, 1),
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn udp_drops_dfs(build: impl BuildFn) -> Result<mc::SearchLog, mc::error::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let collect_log = collect_until_no_events(&mut checker)?;
    println!("Collect finished");
    println!("{collect_log}");
    println!("collected: {}", checker.states_count());

    checker.apply(move |s| s.send_local(&"0:bcast".into(), &messages[0]).unwrap());

    let cfg = mc::SearchConfig::no_faults_with_drops(2);
    let searcher = mc::DfsSearcher::new(cfg);

    checker.check(
        move |s| check_locals_cnt(s.system(), nodes, 1),
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}
