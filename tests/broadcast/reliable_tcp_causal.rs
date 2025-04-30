use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use crate::broadcast::{one_msg, two_msg};

use super::{
    causal::{self, CausalMessage, CausalMessageRegister},
    common::LocalMail,
    connection,
};

////////////////////////////////////////////////////////////////////////////////

struct StreamParser {
    accum: String,
    state: Rc<RefCell<State>>,
}

impl StreamParser {
    fn new(state: Rc<RefCell<State>>) -> Self {
        Self {
            accum: Default::default(),
            state,
        }
    }

    pub fn parse(&mut self, buf: &[u8]) {
        for c in buf.iter().copied() {
            if c == b'\n' {
                let message: CausalMessage = serde_json::from_str(&self.accum).unwrap();
                self.state.borrow_mut().on_msg(message);
                self.accum.clear();
            } else {
                self.accum.push(c as char);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct State {
    senders: BTreeMap<mc::Address, Rc<mc::TcpSender>>,
    msg: BTreeSet<String>,
    reg: Rc<RefCell<CausalMessageRegister>>,
    nodes: usize,
}

impl State {
    fn new(nodes: usize) -> Self {
        Self {
            senders: Default::default(),
            msg: Default::default(),
            reg: Rc::new(RefCell::new(CausalMessageRegister::new(
                nodes,
                LocalMail {},
            ))),
            nodes,
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
        let m = serde_json::to_string(&message).unwrap();
        let count = Rc::new(RefCell::new(1usize)); // 1 for myself (nodes != 1)
        let need = self.need();
        self.senders.values().cloned().for_each(|s| {
            mc::spawn({
                let count = count.clone();
                let mut m = m.clone();
                m.push('\n');
                let message = message.clone();
                let reg = self.reg.clone();
                async move {
                    if s.send(m.as_bytes()).await.is_ok() {
                        let mut x = count.borrow_mut();
                        *x += 1;
                        if *x == need {
                            mc::log("register");
                            reg.borrow_mut().register(message);
                        }
                    }
                }
            });
        });
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Bcast {
    state: Rc<RefCell<State>>,
    nodes: usize,
    me: usize,
}

impl Bcast {
    fn new(nodes: usize, me: usize) -> Self {
        let state = State::new(nodes);
        Self {
            state: Rc::new(RefCell::new(state)),
            nodes,
            me,
        }
    }

    fn connect(&self) {
        (0..self.nodes).filter(|n| *n != self.me).for_each(|n| {
            let addr: mc::Address = format!("{n}:bcast").into();
            let state = self.state.clone();
            mc::spawn(async move {
                let (sender, mut receiver) = connection::connect(addr.clone()).await.split();
                mc::log(format!("connected to {addr}"));
                let insert_result = state.borrow_mut().senders.insert(addr, Rc::new(sender));
                assert!(insert_result.is_none());

                // receive
                let mut parser = StreamParser::new(state);
                let mut buf = [0u8; 1024];
                loop {
                    if let Ok(bytes) = receiver.recv(&mut buf).await {
                        parser.parse(&buf[..bytes]);
                    } else {
                        break;
                    }
                }
            });
        });
    }
}

impl mc::Process for Bcast {
    fn on_message(&mut self, _from: mc::Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        if content == "connect" {
            self.connect();
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
