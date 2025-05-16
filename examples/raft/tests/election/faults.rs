use std::time::Duration;

use raft::{
    addr::{PROCESS_NAME, RAFT_ROLE, make_addr, node},
    proc,
    req::Request,
};

use crate::util::{
    agree_about_leader, concurrent_candidates_appear_count, raft_invariants, some_node_shutdown,
};

////////////////////////////////////////////////////////////////////////////////

fn build_without_fs(s: mc::SystemHandle, nodes: usize) {
    s.network()
        .set_delays(Duration::from_millis(1), Duration::from_millis(10))
        .unwrap();
    (0..nodes).for_each(|n| {
        let mut nd = mc::Node::new(node(n));

        // add proc
        let proc = nd.add_proc(PROCESS_NAME, proc::Raft::default()).unwrap();

        // add node with raft role
        s.add_node_with_role(nd, RAFT_ROLE).unwrap();

        // send init request
        let req = Request::Init { nodes, me: n };
        s.send_local(&proc.address(), req).unwrap();
    });
}

////////////////////////////////////////////////////////////////////////////////

fn build_with_fs(s: mc::SystemHandle, nodes: usize) {
    s.network()
        .set_delays(Duration::from_millis(1), Duration::from_millis(10))
        .unwrap();
    (0..nodes).for_each(|n| {
        let mut nd = mc::Node::new(node(n));

        // add proc
        let proc = nd.add_proc(PROCESS_NAME, proc::Raft::default()).unwrap();

        // add node with raft role
        s.add_node_with_role(nd, RAFT_ROLE).unwrap();

        // setup fs
        s.setup_fs(
            node(n),
            Duration::from_millis(1),
            Duration::from_millis(3),
            4096,
        )
        .unwrap();

        // send init request
        let req = Request::Init { nodes, me: n };
        s.send_local(&proc.address(), req).unwrap();
    });
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_with_faults_no_fs_mc() {
    let nodes = 3;

    // Crash some node in any moment and check election finishes

    let checker = mc::ModelChecker::new_with_build(move |s| build_without_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::with_node_shutdown_only(1));
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 40),
            |s| concurrent_candidates_appear_count(s, 2, 10) > 2,
            move |s| agree_about_leader(s, nodes),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_with_faults_mc() {
    let nodes = 3;

    // Crash some node in any moment and check election finishes

    let checker = mc::ModelChecker::new_with_build(move |s| build_with_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::with_node_shutdown_only(1));
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 40),
            |s| concurrent_candidates_appear_count(s, 2, 10) > 0,
            move |s| agree_about_leader(s, nodes),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_with_node_restart_no_fs_mc() {
    // Nodes are working on leader election...
    // In some moment node shutdown
    // and immediately recovers.
    //
    // After that election must be finished
    // without invariant violation.
    //
    // In this test nodes does not have file system.
    // Because of that, the following situation can occur:
    // 1. Shutdowned node vote for the candidate
    // 2. Candidate receive vote message and become leader
    // 3. Node shutdown
    // 4. The other follower becomes candidate
    // 5. Node receovers and vote for the new candidate
    //   (because vote_for is not saved on the disk as fs unavailable)
    // 6. Candidate receive vote message and become leader
    // 7. We have two leaders in one term. Not good.
    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build_without_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::with_node_shutdown_only(1));

    // Collect all states where nore crashed
    let collect_log = checker
        .collect(
            move |s| raft_invariants(s, nodes, 31),
            |s| {
                s.depth() >= 30
                    || concurrent_candidates_appear_count(s.clone(), 2, 10) > 1
                    || concurrent_candidates_appear_count(s, 3, 10) > 0
            },
            some_node_shutdown,
            searcher,
        )
        .unwrap();
    println!("{collect_log}");

    // Restart shutdown node
    checker.apply(move |s| {
        let mut added = false;
        for id in 0..nodes {
            if s.proc(make_addr(id)).is_none() {
                s.restart_node(node(id)).unwrap();
                s.add_proc_on_node(node(id), PROCESS_NAME, raft::proc::Raft::default())
                    .unwrap();
                let req = Request::Init { nodes, me: id };
                s.send_local(&make_addr(id), req).unwrap();
                added = true;
                break;
            }
        }
        assert!(added);
    });

    // Check leader election finished
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let result = checker.check(
        move |s| raft_invariants(s, nodes, 60),
        move |s| concurrent_candidates_appear_count(s, 2, 100) > 1,
        move |s| agree_about_leader(s, nodes),
        searcher,
    );
    assert!(result.is_err());
    println!("{}", result.unwrap_err());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_with_node_restart_mc() {
    // Nodes are working on leader election...
    // In some moment node shutdown
    // and immediately recovers.
    //
    // After that election must be finished
    // without invariant violation.

    let nodes = 3;
    let mut checker = mc::ModelChecker::new_with_build(move |s| build_with_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::with_node_shutdown_only(1));

    // Collect all states where nore crashed
    let collect_log = checker
        .collect(
            move |s| raft_invariants(s, nodes, 31),
            |s| {
                s.depth() >= 30
                    || concurrent_candidates_appear_count(s.clone(), 2, 100) > 1
                    || concurrent_candidates_appear_count(s, 3, 100) > 0
            },
            |s| {
                some_node_shutdown(s.clone())?;
                agree_about_leader(s, 3)
            },
            searcher,
        )
        .unwrap();
    println!("{collect_log}");

    // Restart shutdown node
    checker.apply(move |s| {
        let mut added = false;
        for id in 0..nodes {
            if s.proc(make_addr(id)).is_none() {
                s.restart_node(node(id)).unwrap();
                s.add_proc_on_node(node(id), PROCESS_NAME, raft::proc::Raft::default())
                    .unwrap();
                let req = Request::Init { nodes, me: id };
                s.send_local(&make_addr(id), req).unwrap();
                added = true;
                break;
            }
        }
        assert!(added);
    });

    // Check leader election finished
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 55),
            move |s| {
                concurrent_candidates_appear_count(s.clone(), 2, 100) > 1
                    || concurrent_candidates_appear_count(s, 3, 100) > 0
            },
            move |s| agree_about_leader(s, nodes),
            searcher,
        )
        .unwrap();

    println!("{log}");
}
