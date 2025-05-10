use std::{cell::RefCell, rc::Rc, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    rpc, send_local, sim::log::RpcMessageDropped, spawn, Address, HashType, LogEntry, Node,
    Process, RpcListener, RpcRequest, RpcResponse, Simulation, StepConfig,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Echo {}

impl From<&RpcRequest> for Echo {
    fn from(value: &RpcRequest) -> Self {
        value.unpack().unwrap()
    }
}

impl From<RpcResponse> for Echo {
    fn from(value: RpcResponse) -> Self {
        value.unpack().unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////

struct RpcSender {
    server: Address,
}

impl Process for RpcSender {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, _content: String) {
        let to = self.server.clone();
        spawn(async {
            let result = rpc(to, 0, &Echo {}).await;
            if let Ok(r) = result {
                let r: Echo = r.into();
                assert_eq!(r, Echo {});
                send_local("ok");
            } else {
                send_local("error");
            }
        });
    }

    fn hash(&self) -> crate::HashType {
        0
    }
}

struct BrokenRpcServer {}

impl Process for BrokenRpcServer {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        assert_eq!(content, "listen");
        let mut listener = RpcListener::register().unwrap();
        spawn(async move {
            loop {
                let r = listener.listen().await;
                let e: Echo = (&r).into();
                assert_eq!(e, Echo {});
                drop(r);
            }
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

#[derive(Default)]
struct WorkingRpcServer {
    request: Rc<RefCell<Option<RpcRequest>>>,
}

impl Process for WorkingRpcServer {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        assert_eq!(content, "listen");
        let mut listener = RpcListener::register().unwrap();
        let request = self.request.clone();
        spawn(async move {
            loop {
                let r = listener.listen().await;
                let e: Echo = (&r).into();
                assert_eq!(e, Echo {});
                *request.borrow_mut() = Some(r);
            }
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn request_not_responsed() {
    let sim = Simulation::new(321);
    let system = sim.system();
    system
        .network()
        .set_delays(Duration::from_millis(100), Duration::from_millis(101))
        .unwrap();

    let mut node = Node::new("n1");
    let p1 = node
        .add_proc(
            "p1",
            RpcSender {
                server: Address::new("n2", "p2"),
            },
        )
        .unwrap();
    system.add_node(node).unwrap();

    let mut node = Node::new("n2");
    let p2 = node.add_proc("p2", BrokenRpcServer {}).unwrap();
    system.add_node(node).unwrap();

    system.send_local(&p2.address(), "listen").unwrap();

    system.send_local(&p1.address(), "123").unwrap();

    sim.step_until_no_events(&StepConfig::no_drops());

    let locals = system.read_locals("n1", "p1").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "error");

    let log = system.log();
    let mut found = false;
    for e in log.iter() {
        if let LogEntry::RpcMessageReceived(r) = e {
            if r.to.node == "n1" {
                assert!(!found);
                found = true;
                assert!(r.time.min() > Duration::from_millis(200));
                assert!(r.time.max() < Duration::from_millis(203));
            }
        }
    }
    assert!(found);

    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn no_listener() {
    let sim = Simulation::new(321);
    let system = sim.system();

    let mut node = Node::new("n1");
    node.add_proc(
        "p1",
        RpcSender {
            server: Address::new("n2", "p2"),
        },
    )
    .unwrap();
    system.add_node(node).unwrap();

    system
        .send_local(&Address::new("n1", "p1"), "lala")
        .unwrap();

    sim.step_until_no_events(&StepConfig::no_drops());

    let log = system.log();
    let mut it = log.iter();
    it.next().unwrap();
    it.next().unwrap();
    let sec = it.next().unwrap();
    assert!(matches!(sec, LogEntry::RpcMessageDropped { .. }));
    let time = variant::variant!(
        sec,
        LogEntry::RpcMessageDropped(RpcMessageDropped { time, .. })
    );
    assert!(time.min() > Duration::ZERO);
    println!("{}", system.log());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn on_node_crash() {
    let sim = Simulation::new(321);
    let system = sim.system();
    system
        .network()
        .set_delays(Duration::from_millis(100), Duration::from_millis(101))
        .unwrap();

    let mut node = Node::new("n1");
    let p1 = node
        .add_proc(
            "p1",
            RpcSender {
                server: Address::new("n2", "p2"),
            },
        )
        .unwrap();
    system.add_node(node).unwrap();

    let mut node = Node::new("n2");
    let p2 = node.add_proc("p2", WorkingRpcServer::default()).unwrap();
    system.add_node(node).unwrap();

    system.send_local(&p2.address(), "listen").unwrap();

    system.send_local(&p1.address(), "123").unwrap();

    sim.step_until_no_events(&StepConfig::no_drops());

    let locals = system.read_locals("n1", "p1").unwrap();
    assert!(locals.is_empty());

    system.crash_node("n2").unwrap();

    sim.step_until_no_events(&StepConfig::no_drops());

    let log = system.log();
    let mut found = false;
    for e in log.iter() {
        if let LogEntry::RpcMessageDropped(r) = e {
            if r.to.node == "n1" {
                assert!(!found);
                found = true;
                assert!(r.time.min() > Duration::from_millis(200));
                assert!(r.time.max() < Duration::from_millis(203));
            }
        }
    }
    assert!(found);

    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn on_sender_crash() {
    let sim = Simulation::new(321);
    let system = sim.system();
    system
        .network()
        .set_delays(Duration::from_millis(100), Duration::from_millis(101))
        .unwrap();

    let mut node = Node::new("n1");
    let p1 = node
        .add_proc(
            "p1",
            RpcSender {
                server: Address::new("n2", "p2"),
            },
        )
        .unwrap();
    system.add_node(node).unwrap();

    let mut node = Node::new("n2");
    let p2 = node.add_proc("p2", WorkingRpcServer::default()).unwrap();
    system.add_node(node).unwrap();

    system.send_local(&p2.address(), "listen").unwrap();

    system.send_local(&p1.address(), "123").unwrap();

    // rpc request delivered
    sim.step(&StepConfig::no_drops());

    system.crash_node("n1").unwrap();

    sim.step_until_no_events(&StepConfig::no_drops());

    let log = system.log();
    println!("{}", log);
}
