use std::time::Duration;

use dsbuild::mc;
use raft::{
    addr::{PROCESS_NAME, RAFT_ROLE, make_addr, node},
    cmd::Command,
    proc::Raft,
    req::Request,
};

use crate::{
    util::{
        agree_about_leader, concurrent_candidates_appear_count, leader, log_equals,
        raft_invariants, some_node_shutdown,
    },
    wrapper::RaftWrapper,
};

////////////////////////////////////////////////////////////////////////////////

fn build_with_fs(sys: dsbuild::model::SystemHandle, nodes: usize) {
    sys.network()
        .set_delays(Duration::from_millis(1), Duration::from_millis(5))
        .unwrap();

    for n in 0..nodes {
        let node_name = node(n);
        let mut node = dsbuild::model::Node::new(&node_name);
        let proc = node.add_proc(PROCESS_NAME, Raft::default()).unwrap();
        sys.add_node_with_role(node, RAFT_ROLE).unwrap();
        sys.setup_fs(
            &node_name,
            Duration::from_millis(1),
            Duration::from_millis(3),
            4096,
        )
        .unwrap();
        sys.send_local(&proc.address(), Request::Init { nodes, me: n })
            .unwrap();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_leader_fault() {
    let raft = RaftWrapper::new(123, 2);
    let leader = raft.step_until_leader_found();
    raft.shutdown_node(leader);

    let response = raft.send_command(leader ^ 1, Command::insert(0, leader ^ 1, "k", "v"));
    assert_eq!(response.id, 0);
    assert!(response.kind.is_err());
    raft.make_many_steps();

    let response = raft.send_command(leader ^ 1, Command::insert(1, leader ^ 1, "k", "v"));
    assert_eq!(response.id, 1);
    assert!(response.kind.is_err());

    raft.restart_node(leader);
    let leader = raft.step_until_leader_found();

    let response = raft.send_command(leader, Command::insert(2, leader, "k", "v"));
    assert_eq!(response.id, 2);
    assert!(response.kind.is_ok());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_leader_fault_mc() {
    let nodes = 2;

    let mut checker = mc::ModelChecker::new_with_build(move |s| build_with_fs(s, nodes));

    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let collect_log = checker
        .collect(
            move |s| raft_invariants(s, nodes, 31),
            |s| {
                concurrent_candidates_appear_count(s.system(), 2, 100) > 1
                    || concurrent_candidates_appear_count(s.system(), 3, 100) > 0
                    || s.depth() > 30
            },
            move |s| agree_about_leader(s.system(), nodes),
            searcher,
        )
        .unwrap();

    println!("{collect_log}");
    println!("Collected where leader elected: {}", checker.states_count());

    checker.apply(move |s| {
        let leader = leader(s.clone(), nodes).unwrap().unwrap();
        s.send_local(
            &make_addr(leader as usize),
            Request::Command(Command::insert(0, leader as usize, "k", "v")),
        )
        .unwrap()
    });

    // collect states where leader saved log
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let collect_log = checker
        .collect(
            move |s| raft_invariants(s, nodes, 40),
            |s| {
                concurrent_candidates_appear_count(s.system(), 2, 100) > 1
                    || concurrent_candidates_appear_count(s.system(), 3, 100) > 0
            },
            move |s| {
                let leader = leader(s.system(), nodes).unwrap().unwrap() as usize;
                let state = s.system().proc_state::<Raft>(make_addr(leader)).unwrap();
                let state = state.borrow().handle().unwrap();
                let last_log_index = state.last_log_index();
                if last_log_index == 1 {
                    Ok(())
                } else {
                    Err(format!("last log index is {last_log_index}"))
                }
            },
            searcher,
        )
        .unwrap();

    println!("{collect_log}");
    println!(
        "Collected where leader saved log: {}",
        checker.states_count()
    );

    // collect states where someone fault
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::with_node_shutdown_only(1));
    let collect_log = checker
        .collect(
            move |s| raft_invariants(s, nodes, 51),
            |s| {
                concurrent_candidates_appear_count(s.system(), 2, 100) > 1
                    || concurrent_candidates_appear_count(s.system(), 3, 100) > 0
                    || s.depth() > 50
            },
            move |s| some_node_shutdown(s.system()),
            searcher,
        )
        .unwrap();

    println!("{collect_log}");
    println!(
        "Collected where some node shutdown: {}",
        checker.states_count()
    );

    // restart shutdown node
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

    // check log replicated

    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 60),
            |s| {
                concurrent_candidates_appear_count(s.system(), 2, 100) > 1
                    || concurrent_candidates_appear_count(s.system(), 3, 100) > 0
            },
            move |s| {
                let log = log_equals(s.system(), nodes)?.ok_or("not found log")?;
                if log.len() == 1 {
                    Ok(())
                } else {
                    Err("log len must be 1".into())
                }
            },
            searcher,
        )
        .unwrap();

    println!("{log}");
}
