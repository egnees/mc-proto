use std::time::Duration;

use mc::StepConfig;
use raft::{
    addr::{PROCESS_NAME, RAFT_ROLE, make_addr, node},
    cmd::{Command, CommandKind, Response},
    proc::Raft,
    req::Request,
};

use crate::util::leader;

////////////////////////////////////////////////////////////////////////////////

fn build_with_fs(sys: mc::SystemHandle, nodes: usize) {
    for n in 0..nodes {
        let node_name = node(n);
        let mut node = mc::Node::new(&node_name);
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

    let sim = mc::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = StepConfig::no_drops();
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

    println!("{}", sys.log());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_send_to_leader() {
    let nodes = 2;

    let sim = mc::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = StepConfig::no_drops();
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
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_nodes_send_to_not_leader() {
    let nodes = 2;

    let sim = mc::Simulation::new(123);
    let sys = sim.system();
    build_with_fs(sys.clone(), nodes);

    let cfg = StepConfig::no_drops();
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
    }
}
