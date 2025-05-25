use std::{cell::RefCell, rc::Rc, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    mc::{
        search::{gen::Generator, state::SearchState},
        SearchConfig,
    },
    model::{event::driver::EventDriver, net::Config as NetConfig, node::Node, system::System},
    rpc, send_local, spawn, Address, HashType, Process, RpcListener,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Echo {}

////////////////////////////////////////////////////////////////////////////////

struct RpcSender {
    to: Address,
}

impl Process for RpcSender {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, _content: String) {
        let to = self.to.clone();
        spawn(async move {
            let response = rpc(to, 0, &Echo {}).await.unwrap();
            let echo = response.unpack::<Echo>().unwrap();
            assert_eq!(echo, Echo {});
            send_local("echo!");
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

struct RpcServer {}

impl Process for RpcServer {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        assert_eq!(content, "listen");
        let mut listener = RpcListener::register().unwrap();
        spawn(async move {
            loop {
                let req = listener.listen().await;
                let echo = req.unpack::<Echo>().unwrap();
                assert_eq!(echo, Echo {});
                req.reply(&Echo {}).unwrap();
            }
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic() {
    // build system
    let gen = Rc::new(RefCell::new(Generator::new()));
    let net = NetConfig::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap();
    let system = System::new(&net, &(gen.clone() as Rc<RefCell<dyn EventDriver>>));
    let mut state = SearchState { system, gen };

    let system = state.system.handle();

    let mut node = Node::new("s");
    let server = node.add_proc("s", RpcServer {}).unwrap();
    system.add_node(node).unwrap();

    let mut node = Node::new("c");
    let sender = node
        .add_proc(
            "c",
            RpcSender {
                to: server.address(),
            },
        )
        .unwrap();
    system.add_node(node).unwrap();

    system.send_local(&server.address(), "listen").unwrap();
    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert!(steps.is_empty());

    system.send_local(&sender.address(), "123").unwrap();
    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert_eq!(steps.len(), 1);
    steps[0].apply(&mut state).unwrap();

    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert_eq!(steps.len(), 1);
    steps[0].apply(&mut state).unwrap();

    let locals = system.read_locals("c", "c").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "echo!");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_sends() {
    // build system
    let gen = Rc::new(RefCell::new(Generator::new()));
    let net = NetConfig::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap();
    let system = System::new(&net, &(gen.clone() as Rc<RefCell<dyn EventDriver>>));
    let mut state = SearchState { system, gen };

    let system = state.system.handle();

    let mut node = Node::new("s");
    let server = node.add_proc("s", RpcServer {}).unwrap();
    system.add_node(node).unwrap();

    let mut node = Node::new("c");
    let sender = node
        .add_proc(
            "c",
            RpcSender {
                to: server.address(),
            },
        )
        .unwrap();
    system.add_node(node).unwrap();

    system.send_local(&server.address(), "listen").unwrap();
    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert!(steps.is_empty());

    system.send_local(&sender.address(), "123").unwrap();
    system.send_local(&sender.address(), "321").unwrap();
    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert_eq!(steps.len(), 1);
    steps[0].apply(&mut state).unwrap();

    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert_eq!(steps.len(), 2);
    steps[1].apply(&mut state).unwrap();

    let locals = system.read_locals("c", "c").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "echo!");

    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert_eq!(steps.len(), 1);
    steps[0].apply(&mut state).unwrap();

    let steps = state.steps(&SearchConfig::no_faults_no_drops());
    assert_eq!(steps.len(), 1);
    steps[0].apply(&mut state).unwrap();

    let locals = system.read_locals("c", "c").unwrap();
    assert_eq!(locals.len(), 2);
    assert_eq!(locals[1], "echo!");
}
