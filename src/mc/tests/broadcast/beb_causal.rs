use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::{
    log,
    model::{node::Node, tcp::TcpError, SystemHandle},
    send_local, Address, HashType, Process,
};

use super::two_msg::{self};

use super::{
    causal::{make_message, CausalMessage, CausalMessageRegister},
    common::LocalMail,
    connection, one_msg,
};

use crate::prelude::*;

////////////////////////////////////////////////////////////////////////////////
/// Best Effort Broadcast
////////////////////////////////////////////////////////////////////////////////

struct StreamParser {
    accum: String,
    reg: Rc<RefCell<CausalMessageRegister>>,
}

impl StreamParser {
    fn new(reg: Rc<RefCell<CausalMessageRegister>>) -> Self {
        Self {
            accum: Default::default(),
            reg,
        }
    }

    fn parse(&mut self, buf: &[u8]) {
        for c in buf.iter().copied() {
            if c == b'\n' {
                let msg: CausalMessage = serde_json::from_str(self.accum.as_str())
                    .expect("can not deserialize in parser");
                self.reg.borrow_mut().register(msg);
                self.accum.clear();
            } else {
                self.accum.push(c as char);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

async fn communicate_with(
    with: Address,
    mut receiver: UnboundedReceiver<CausalMessage>,
    reg: Rc<RefCell<CausalMessageRegister>>,
) -> Result<(), TcpError> {
    let mut stream = connection::connect(with).await;
    log(format!("connected to {}", stream.to()));
    let mut buf = [0u8; 1024];
    let mut parser = StreamParser::new(reg);
    loop {
        tokio::select! {
            from_other = stream.recv(&mut buf) => {
                let bytes = from_other?;
                parser.parse(&buf[..bytes]);
            }
            from_user = receiver.recv() => {
                if let Some(msg) = from_user {
                    let mut msg = serde_json::to_string(&msg).unwrap();
                    msg.push('\n');
                    stream.send(msg.as_str().as_bytes()).await?;
                } else {
                    break;
                }
            }
        }
    }
    loop {
        let bytes = stream.recv(&mut buf).await?;
        parser.parse(&buf[..bytes]);
    }
}

////////////////////////////////////////////////////////////////////////////////

struct BebProcess {
    proc: Vec<Address>,
    senders: HashMap<Address, UnboundedSender<CausalMessage>>,
    locals: Vec<String>,
    me: usize,
    reg: Rc<RefCell<CausalMessageRegister>>,
}

impl BebProcess {
    fn new(others: usize, me: usize) -> Self {
        let mail = LocalMail {};
        Self {
            proc: (0..others)
                .map(|n| format!("{n}:bcast").into())
                .collect::<Vec<_>>(),
            senders: Default::default(),
            locals: Default::default(),
            me,
            reg: Rc::new(RefCell::new(CausalMessageRegister::new(others, mail))),
        }
    }

    fn iter_others(&self) -> impl Iterator<Item = &Address> {
        (0..self.proc.len())
            .filter(|i| *i != self.me)
            .map(|i| self.proc.get(i).unwrap())
    }
}

impl Process for BebProcess {
    fn on_message(&mut self, _from: Address, content: String) {
        send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        if content != "connect" {
            self.locals.push(content.clone());
        }

        if self.senders.is_empty() {
            let others = self.iter_others().cloned().collect::<Vec<_>>();
            others.into_iter().for_each(|other| {
                let (sender, receiver) = unbounded_channel();
                self.senders.insert(other.clone(), sender);
                spawn(communicate_with(other, receiver, self.reg.clone()));
            });
        }

        if content != "connect" {
            let msg = make_message(content, self.me, self.reg.borrow().vc().clone());

            self.reg.borrow_mut().register(msg.clone());

            self.iter_others()
                .map(|other| self.senders.get(other).unwrap())
                .for_each(|s| {
                    let _ = s.send(msg.clone());
                });
        }
    }

    fn hash(&self) -> HashType {
        let mut hasher = DefaultHasher::new();
        self.reg.borrow().hash(&mut hasher);
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn build(s: SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = BebProcess::new(nodes, proc);
        let mut node = Node::new(node_name);
        let proc_handle = node.add_proc("bcast", proc).unwrap();
        s.add_node(node).unwrap();
        s.send_local(&proc_handle.address(), "connect").unwrap();
    });
}

pub fn build_with_roles(s: SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = BebProcess::new(nodes, proc);
        let mut node = Node::new(node_name);
        let proc_handle = node.add_proc("bcast", proc).unwrap();
        s.add_node_with_role(node, "bcast").unwrap();
        s.send_local(&proc_handle.address(), "connect").unwrap();
    });
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_message_no_faults() {
    let log = one_msg::no_drops(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
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
fn two_locals_no_faults_check_causal() {
    let log = two_msg::same_node_no_drop_no_fault_check_causal(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_locals_concurrent_no_faults_check_causal() {
    let log = two_msg::concurrent_no_drop_no_fault_check_causal(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_locals_sequenced_no_faults_check_causal() {
    let log = two_msg::send_after_recv_no_drop_no_fault_check_causal(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_locals_sequenced_no_faults_check_all() {
    let log = two_msg::send_after_recv_no_drop_no_fault_check_all(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_locals_sequenced_no_faults_check_all_with_roles() {
    let log = two_msg::send_after_recv_no_drop_no_fault_check_all(build_with_roles).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn two_locals_sequenced_with_faults_check_all() {
    let log = two_msg::send_after_recv_no_drop_with_fault_check_all(build).unwrap();
    println!("{log}");
}
