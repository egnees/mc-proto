use std::time::Duration;

use raft::{
    addr::{PROCESS_NAME, RAFT_ROLE, node},
    proc::{self},
    req::Request,
};

use crate::{
    util::{
        agree_about_leader, concurrent_candidates_appear_count, no_two_leaders_in_one_term,
        raft_invariants,
    },
    wrapper::RaftWrapper,
};

////////////////////////////////////////////////////////////////////////////////

fn build_with_fs(s: mc::SystemHandle, nodes: usize) {
    s.network()
        .set_delays(Duration::from_millis(1), Duration::from_millis(5))
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

fn build_without_fs(s: mc::SystemHandle, nodes: usize) {
    s.network()
        .set_delays(Duration::from_millis(1), Duration::from_millis(5))
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

#[test]
fn one_node_basic_mc() {
    let checker = mc::ModelChecker::new_with_build(|s| build_with_fs(s, 1));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            |_| Ok(()),
            |_| false,
            |s| agree_about_leader(s.system(), 1),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_basic_no_fs_mc() {
    let nodes = 2;
    let checker = mc::ModelChecker::new_with_build(move |s| build_without_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| no_two_leaders_in_one_term(s.system(), nodes),
            move |s| concurrent_candidates_appear_count(s.system(), nodes, 10) > 2,
            move |s| agree_about_leader(s.system(), nodes),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_basic_mc() {
    let nodes = 2;
    let checker = mc::ModelChecker::new_with_build(move |s| build_with_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| no_two_leaders_in_one_term(s.system(), nodes),
            move |s| concurrent_candidates_appear_count(s.system(), nodes, 100) > 0,
            move |s| agree_about_leader(s.system(), nodes),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_basic_no_fs_mc() {
    let nodes = 3;
    let checker = mc::ModelChecker::new_with_build(move |s| build_without_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 30),
            move |s| concurrent_candidates_appear_count(s.system(), nodes, 10) > 1,
            move |s| agree_about_leader(s.system(), nodes),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_basic_mc() {
    let nodes = 3;
    let checker = mc::ModelChecker::new_with_build(move |s| build_with_fs(s, nodes));
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 40),
            move |s| concurrent_candidates_appear_count(s.system(), 2, 100) > 1,
            move |s| agree_about_leader(s.system(), nodes),
            searcher,
        )
        .unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn five_nodes_election_chaos() {
    let nodes = 5;
    for seed in 0..250 {
        let raft = RaftWrapper::new(seed, nodes);
        raft.step_until_leader_found();
    }
}
