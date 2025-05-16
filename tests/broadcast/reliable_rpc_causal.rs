use std::{
    cell::RefCell,
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use mc::{rpc, Address};
use serde::{Deserialize, Serialize};

use crate::broadcast::{one_msg, two_msg};

use super::{
    causal::{self, CausalMessage, CausalMessageRegister},
    common::LocalMail,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
struct RpcResponse;

////////////////////////////////////////////////////////////////////////////////

struct State {
    nodes: usize,
    me: usize,
    msg: BTreeSet<String>,
    reg: Rc<RefCell<CausalMessageRegister>>,
}

impl State {
    fn new(nodes: usize, me: usize) -> Self {
        Self {
            nodes,
            me,
            msg: Default::default(),
            reg: Rc::new(RefCell::new(CausalMessageRegister::new(
                nodes,
                LocalMail {},
            ))),
        }
    }

    fn need(&self) -> usize {
        self.nodes / 2 + 1
    }

    fn on_msg(&mut self, message: CausalMessage) {
        let prev = self.msg.insert(message.content.clone());
        if !prev {
            return;
        }
        self.bcast_message(message);
    }

    fn bcast_message(&self, message: CausalMessage) {
        let count = Rc::new(RefCell::new(1usize)); // 1 for myself (nodes != 1)
        let need = self.need();
        for node in 0..self.nodes {
            if node == self.me {
                continue;
            }
            let addr: mc::Address = format!("{node}:bcast").into();
            mc::spawn({
                let msg = message.clone();
                let count = count.clone();
                let reg = self.reg.clone();
                async move {
                    let result = rpc(addr, 0, &msg).await;
                    if result.is_ok() {
                        let mut x = count.borrow_mut();
                        *x += 1;
                        if *x == need {
                            mc::log("register");
                            reg.borrow_mut().register(msg);
                        }
                    }
                }
            });
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Bcast {
    me: usize,
    state: Rc<RefCell<State>>,
}

impl Bcast {
    fn new(nodes: usize, me: usize) -> Self {
        let state = State::new(nodes, me);
        Self {
            me,
            state: Rc::new(RefCell::new(state)),
        }
    }
}

impl mc::Process for Bcast {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        if content == "connect" {
            let state = self.state.clone();
            let mut listener = mc::RpcListener::register().unwrap();
            mc::spawn(async move {
                loop {
                    let request = listener.listen().await;
                    let msg: CausalMessage = request.unpack().unwrap();
                    state.borrow_mut().on_msg(msg);
                    let _ = request.reply(&RpcResponse);
                }
            });
        } else {
            let vc = self.state.borrow().reg.borrow_mut().vc().clone();
            let message = causal::make_message(content, self.me, vc);
            self.state.borrow_mut().on_msg(message);
        }
    }

    fn hash(&self) -> mc::HashType {
        let mut hasher = DefaultHasher::new();
        self.state.borrow().reg.borrow().hash(&mut hasher);
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn build(s: mc::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = Bcast::new(nodes, proc);
        let mut node = mc::Node::new(node_name);
        let proc = node.add_proc("bcast", proc).unwrap();
        s.add_node_with_role(node, "bcast").unwrap();
        s.send_local(&proc.address(), "connect").unwrap();
    });
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_message_no_faults() {
    let log = one_msg::no_drops(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_message_node_crash() {
    let log = one_msg::node_crash_after_someone_delivery(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_message_udp_drop_bfs() {
    let log = one_msg::udp_drops_bfs(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_message_udp_drop_dfs() {
    let log = one_msg::udp_drops_dfs(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_sequenced_messages_without_faults() {
    let log = two_msg::send_after_recv_no_drop_no_fault_check_all(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_sequenced_messages_with_faults() {
    let log = two_msg::send_after_recv_no_drop_with_fault_check_all(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(not(debug_assertions))]
#[test]
fn two_concurrent_messages_with_faults() {
    let log = two_msg::concurrent_with_faults_check_validity_and_agreement(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(not(debug_assertions))]
#[test]
fn two_concurrent_messages_without_faults() {
    let log = two_msg::concurrent_without_faults_check_validity_and_agreement(build).unwrap();
    println!("{}", log);
}
