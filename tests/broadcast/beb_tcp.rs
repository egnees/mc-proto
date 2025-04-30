use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::broadcast::two_msg::{self};

use super::{connection, one_msg};

////////////////////////////////////////////////////////////////////////////////
/// Best Effort Broadcast
////////////////////////////////////////////////////////////////////////////////

struct StreamParser {
    accum: String,
    reg: Rc<RefCell<Vec<String>>>,
}

impl StreamParser {
    pub fn parse(&mut self, buf: &[u8]) {
        for c in buf.iter().copied() {
            if c == b'\n' {
                self.reg.borrow_mut().push(self.accum.clone());
                mc::send_local(&self.accum);
                self.accum.clear();
            } else {
                self.accum.push(c as char);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

async fn communicate_with(
    with: mc::Address,
    mut receiver: UnboundedReceiver<String>,
    reg: Rc<RefCell<Vec<String>>>,
) -> Result<(), mc::TcpError> {
    let mut stream = connection::connect(with).await;
    let mut buf = [0u8; 256];
    let mut parser = StreamParser {
        accum: Default::default(),
        reg,
    };
    loop {
        tokio::select! {
            from_other = stream.recv(&mut buf) => {
                let bytes = from_other?;
                parser.parse(&buf[..bytes]);
            }
            from_user = receiver.recv() => {
                if let Some(mut msg) = from_user {
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
    proc: Vec<mc::Address>,
    senders: HashMap<mc::Address, UnboundedSender<String>>,
    reg: Rc<RefCell<Vec<String>>>,
    me: usize,
}

impl BebProcess {
    fn new(others: usize, me: usize) -> Self {
        Self {
            proc: (0..others)
                .map(|n| format!("{n}:bcast").into())
                .collect::<Vec<_>>(),
            senders: Default::default(),
            reg: Rc::new(RefCell::new(Default::default())),
            me,
        }
    }

    fn iter_others(&self) -> impl Iterator<Item = &mc::Address> {
        (0..self.proc.len())
            .filter(|i| *i != self.me)
            .map(|i| self.proc.get(i).unwrap())
    }
}

impl mc::Process for BebProcess {
    fn on_message(&mut self, _from: mc::Address, content: String) {
        mc::send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        if content != "connect" {
            self.reg.borrow_mut().push(content.clone());
        }

        if self.senders.is_empty() {
            let others = self.iter_others().cloned().collect::<Vec<_>>();
            others.into_iter().for_each(|other| {
                let (sender, receiver) = unbounded_channel();
                self.senders.insert(other.clone(), sender);
                mc::spawn(communicate_with(other, receiver, self.reg.clone()));
            });
        }

        if content != "connect" {
            self.iter_others()
                .map(|other| self.senders.get(other).unwrap())
                .for_each(|s| {
                    let _ = s.send(content.clone());
                });

            mc::send_local(content);
        }
    }

    fn hash(&self) -> mc::HashType {
        let mut hasher = DefaultHasher::new();
        self.reg.borrow().iter().for_each(|s| s.hash(&mut hasher));
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn build(s: mc::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = BebProcess::new(nodes, proc);
        let mut node = mc::Node::new(node_name);
        let proc_handle = node.add_proc("bcast", proc).unwrap();
        s.add_node(node).unwrap();
        s.send_local(&proc_handle.address(), "connect").unwrap();
    });
}

////////////////////////////////////////////////////////////////////////////////

pub fn build_with_roles(s: mc::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = BebProcess::new(nodes, proc);
        let mut node = mc::Node::new(node_name);
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

#[test]
fn one_message_with_roles_no_faults() {
    let log = one_msg::no_drops(build_with_roles).unwrap();
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
fn two_locals_same_node_no_faults_check_causal() {
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

#[should_panic]
#[test]
fn two_locals_sequenced_no_faults_check_causal() {
    let log = two_msg::send_after_recv_no_drop_no_fault_check_causal(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn two_locals_concurrent_node_fail() {
    let log = two_msg::concurrent_with_faults_check_validity_and_agreement(build).unwrap();
    println!("{log}");
}
