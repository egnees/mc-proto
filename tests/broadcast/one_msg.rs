use crate::broadcast::check::check_depth;

use super::{
    check::{check_locals_cnt, check_someone_deliver, check_validity_and_agreement},
    common::{send_local, BuildFn},
};

////////////////////////////////////////////////////////////////////////////////

pub fn no_drops(build: impl BuildFn) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let build = {
        let loc = messages[0].clone();
        move |s: mc::SystemHandle| {
            build(s.clone(), nodes);
            let send_result = send_local(s, 0, &loc);
            assert!(send_result);
        }
    };
    let checker = mc::ModelChecker::new_with_build(build);
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);

    checker.check(
        |_| Ok(()),
        // prune: we must find state with depth <= 10 which delivers message
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn node_crash_after_someone_delivery(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let build = {
        let loc = messages[0].clone();
        move |s: mc::SystemHandle| {
            build(s.clone(), nodes);
            let send_result = send_local(s, 0, &loc);
            assert!(send_result);
        }
    };

    let mut checker = mc::ModelChecker::new_with_build(build);
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);
    let collect_log = checker.collect(
        |s| check_depth(s, 20),
        |_| false,
        move |s| {
            check_someone_deliver(s.system(), nodes)?;
            Ok(())
        },
        searcher,
    )?;

    println!("Collect log:");
    println!("{}", collect_log);

    let cfg = mc::SearchConfig::with_node_faults_only(1);
    let searcher = mc::BfsSearcher::new(cfg);
    checker.check(
        move |s| check_depth(s, 20),
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn udp_drops_bfs(build: impl BuildFn) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let build = {
        let loc = messages[0].clone();
        move |s: mc::SystemHandle| {
            build(s.clone(), nodes);
            let send_result = send_local(s, 0, &loc);
            assert!(send_result);
        }
    };
    let checker = mc::ModelChecker::new_with_build(build);
    let cfg = mc::SearchConfig::no_faults_with_drops(2);
    let searcher = mc::BfsSearcher::new(cfg);

    checker.check(
        |_| Ok(()),
        // prune: we must find state with depth <= 20 which delivers message
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn udp_drops_dfs(build: impl BuildFn) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let build = {
        let loc = messages[0].clone();
        move |s: mc::SystemHandle| {
            build(s.clone(), nodes);
            let send_result = send_local(s, 0, &loc);
            assert!(send_result);
        }
    };
    let checker = mc::ModelChecker::new_with_build(build);
    let cfg = mc::SearchConfig::no_faults_with_drops(2);
    let searcher = mc::DfsSearcher::new(cfg);

    checker.check(
        |_| Ok(()),
        // prune: we must find state with depth <= 20 which delivers message
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}

////////////////////////////////////////////////////////////////////////////////

pub fn no_drops_no_faults_check_no_duplications(
    build: impl BuildFn,
) -> Result<mc::SearchLog, mc::SearchError> {
    let nodes = 3;
    let messages = vec!["0:Hello".to_string()];
    let build = {
        let loc = messages[0].clone();
        move |s: mc::SystemHandle| {
            build(s.clone(), nodes);
            let send_result = send_local(s, 0, &loc);
            assert!(send_result);
        }
    };
    let checker = mc::ModelChecker::new_with_build(build);
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);

    checker.check(
        move |s| check_locals_cnt(s, nodes, 1),
        // prune: we must find state with depth <= 10 which delivers message
        |_| false,
        move |s| check_validity_and_agreement(s.system(), nodes),
        searcher,
    )
}
