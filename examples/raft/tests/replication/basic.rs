use std::time::Duration;

use dsbuild::{detsim, mc};
use raft::{
    addr::{PROCESS_NAME, RAFT_ROLE, make_addr, node},
    cmd::{Command, CommandKind, Response, ResponseKind},
    proc::Raft,
    req::Request,
};

use crate::{
    util::{
        agree_about_leader, concurrent_candidates_appear_count, leader, log_equals, raft_invariants,
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
fn one_node() {
    let nodes = 1;

    let sim = detsim::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = detsim::StepConfig::no_drops();
    sim.step_unti(|s| leader(s, nodes).is_ok(), &cfg);

    // initialized

    sys.send_local(
        &make_addr(0),
        Request::Command(Command {
            id: 0,
            leader: 0,
            kind: CommandKind::Insert {
                key: "k".into(),
                value: "v".into(),
            },
        }),
    )
    .unwrap();

    for _ in 0..20 {
        sim.step(&cfg);
    }

    // command must be applied

    let addr = make_addr(0);
    let locals = sys.read_locals(addr.node, addr.process).unwrap();
    assert_eq!(locals.len(), 1);
    let resp: Response = locals[0].clone().into();
    assert_eq!(resp.id, 0);
    assert!(resp.kind.is_ok());

    let log = log_equals(sys.clone(), nodes).unwrap().unwrap();
    assert_eq!(log.len(), 1);

    println!("{}", sys.log());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_send_to_leader() {
    let nodes = 2;

    let sim = detsim::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = detsim::StepConfig::no_drops();
    sim.step_unti(|s| leader(s, nodes).is_ok_and(|l| l.is_some()), &cfg);

    let leader = leader(sys.clone(), nodes).unwrap().unwrap() as usize;

    // initialized

    sys.send_local(
        &make_addr(leader),
        Request::Command(Command {
            id: 0,
            leader,
            kind: CommandKind::Insert {
                key: "k".into(),
                value: "v".into(),
            },
        }),
    )
    .unwrap();

    for _ in 0..10 {
        sim.step(&cfg);
    }

    // command must be applied

    let addr = make_addr(leader);
    let locals = sys.read_locals(addr.node, addr.process).unwrap();
    assert_eq!(locals.len(), 1);
    let resp: Response = locals[0].clone().into();
    assert_eq!(resp.id, 0);
    assert!(resp.kind.is_ok());

    println!("{}", sys.log());

    let log = log_equals(sys.clone(), nodes).unwrap().unwrap();
    assert_eq!(log.len(), 1);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_send_to_not_leader() {
    let nodes = 2;

    let sim = detsim::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = detsim::StepConfig::no_drops();
    sim.step_unti(|s| leader(s, nodes).is_ok_and(|l| l.is_some()), &cfg);

    let not_leader = leader(sys.clone(), nodes).unwrap().unwrap() ^ 1;
    let not_leader = not_leader as usize;

    // initialized

    sys.send_local(
        &make_addr(not_leader),
        Request::Command(Command {
            id: 0,
            leader: not_leader,
            kind: CommandKind::Insert {
                key: "k".into(),
                value: "v".into(),
            },
        }),
    )
    .unwrap();

    for _ in 0..10 {
        sim.step(&cfg);
    }

    let addr = make_addr(not_leader);
    let locals = sys.read_locals(addr.node, addr.process).unwrap();
    assert_eq!(locals.len(), 1);

    let resp: Response = locals[0].clone().into();
    assert_eq!(resp.id, 0);
    let err = resp.kind.unwrap_err();
    match err {
        raft::cmd::Error::NotLeader { redirected_to } => {
            assert_eq!(redirected_to, Some(not_leader ^ 1))
        }
        _ => unreachable!(),
    }

    let log = log_equals(sys.clone(), nodes).unwrap().unwrap();
    assert!(log.is_empty());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_send_to_leader() {
    let nodes = 3;

    let sim = detsim::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = detsim::StepConfig::no_drops();
    sim.step_unti(|s| leader(s, nodes).is_ok_and(|l| l.is_some()), &cfg);

    let leader = leader(sys.clone(), nodes).unwrap().unwrap() as usize;

    // initialized

    sys.send_local(
        &make_addr(leader),
        Request::Command(Command {
            id: 0,
            leader,
            kind: CommandKind::Insert {
                key: "k".into(),
                value: "v".into(),
            },
        }),
    )
    .unwrap();

    for _ in 0..20 {
        sim.step(&cfg);
    }

    // command must be applied

    let addr = make_addr(leader);
    let locals = sys.read_locals(addr.node, addr.process).unwrap();
    assert_eq!(locals.len(), 1);
    let resp: Response = locals[0].clone().into();
    assert_eq!(resp.id, 0);
    assert!(resp.kind.is_ok());

    println!("{}", sys.log());

    let log = log_equals(sys, nodes).unwrap().unwrap();
    assert_eq!(log.len(), 1);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_send_to_not_leader() {
    let nodes = 3;

    let sim = detsim::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = detsim::StepConfig::no_drops();
    sim.step_unti(|s| leader(s, nodes).is_ok_and(|l| l.is_some()), &cfg);

    let not_leader = leader(sys.clone(), nodes).unwrap().unwrap() ^ 1;
    let not_leader = not_leader as usize;

    // initialized

    sys.send_local(
        &make_addr(not_leader),
        Request::Command(Command {
            id: 0,
            leader: not_leader,
            kind: CommandKind::Insert {
                key: "k".into(),
                value: "v".into(),
            },
        }),
    )
    .unwrap();

    for _ in 0..10 {
        sim.step(&cfg);
    }

    let addr = make_addr(not_leader);
    let locals = sys.read_locals(addr.node, addr.process).unwrap();
    assert_eq!(locals.len(), 1);

    let resp: Response = locals[0].clone().into();
    assert_eq!(resp.id, 0);
    let err = resp.kind.unwrap_err();
    match err {
        raft::cmd::Error::NotLeader { redirected_to } => {
            assert_eq!(redirected_to, Some(not_leader ^ 1))
        }
        _ => unreachable!(),
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_seq_to_leader_chaos() {
    for seed in 0..250 {
        let raft = RaftWrapper::new(seed, 3);
        let leader = raft.step_until_leader_found();

        // insert (k, v)
        let resp = raft.send_command(leader, Command::insert(0, leader, "k", "v"));
        assert_eq!(resp.id, 0);
        assert!(resp.kind.is_ok());
        assert_eq!(resp.kind.unwrap(), ResponseKind::Insert { prev: None });

        // read k
        let resp = raft.send_command(leader, Command::read(1, leader, "k"));
        assert_eq!(resp.id, 1);
        assert!(resp.kind.is_ok());
        assert_eq!(
            resp.kind.unwrap(),
            ResponseKind::Read {
                value: Some("v".into())
            }
        );

        // CAS
        let resp = raft.send_command(leader, Command::cas(2, leader, "k", "v", "v1"));

        assert_eq!(resp.id, 2);
        assert!(resp.kind.is_ok());
        assert_eq!(resp.kind.unwrap(), ResponseKind::CAS { complete: true });

        let log = raft.step_until_log_equals();
        assert_eq!(log.len(), 3);
    }
}

////////////////////////////////////////////////////////////////////////////////

fn template_mc(nodes: usize) {
    let mut checker = mc::ModelChecker::new_with_build(move |s| build_with_fs(s, nodes));

    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let collect_log = checker
        .collect(
            move |s| raft_invariants(s, nodes, 45),
            move |s| concurrent_candidates_appear_count(s.system(), 2, 10) > 1,
            move |s| agree_about_leader(s.system(), nodes),
            searcher,
        )
        .unwrap();

    println!("{collect_log}");
    println!("Collected: {}", checker.states_count());

    checker.apply(move |s| {
        let leader = leader(s.clone(), nodes).unwrap().unwrap();
        s.send_local(
            &make_addr(leader as usize),
            Request::Command(Command::insert(0, leader as usize, "k", "v")),
        )
        .unwrap()
    });

    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let log = checker
        .check(
            move |s| raft_invariants(s, nodes, 45),
            |_| false,
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

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_node_mc() {
    template_mc(1);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_mc() {
    template_mc(2);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn three_nodes_mc() {
    template_mc(3);
}
