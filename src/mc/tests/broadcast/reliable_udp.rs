use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};

pub use crate::prelude::*;
use crate::{
    mc::tests::broadcast::{one_msg, two_msg},
    model::{self, net::send_message},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Bcast(String),
    Ack(String),
}

////////////////////////////////////////////////////////////////////////////////

pub struct Bcast {
    proc_cnt: usize,
    me: usize,
    reg: BTreeSet<String>,
    pending: BTreeMap<String, usize>,
}

impl Bcast {
    fn new(nodes: usize, me: usize) -> Self {
        Self {
            proc_cnt: nodes,
            me,
            reg: Default::default(),
            pending: Default::default(),
        }
    }

    fn others<'a>(&'a self) -> impl Iterator<Item = Address> + 'a {
        (0..self.proc_cnt)
            .filter(|x| *x != self.me)
            .map(|node| format!("{node}:bcast").into())
    }

    fn ack(&mut self, on: String) {
        let v = self.pending.entry(on.clone()).or_insert(0);
        *v += 1;
        if *v == self.proc_cnt.div_ceil(2) {
            send_local(on);
        }
    }
}

impl Process for Bcast {
    fn on_message(&mut self, from: Address, orign_content: String) {
        let msg: Message = serde_json::from_str(orign_content.as_str()).unwrap();
        match msg {
            Message::Bcast(content) => {
                let insert_result = self.reg.insert(content.clone());

                // bcast further
                if insert_result {
                    // for myself
                    self.ack(content.clone());

                    // for sender
                    self.ack(content.clone());

                    for other in self.others() {
                        if other != from {
                            send_message(&other, orign_content.clone());
                        }
                    }
                }

                // send ack
                let msg = Message::Ack(content);
                let msg = serde_json::to_string(&msg).unwrap();
                send_message(&from, msg);
            }
            Message::Ack(content) => {
                self.ack(content);
            }
        }
    }

    fn on_local_message(&mut self, content: String) {
        self.reg.insert(content.clone());
        self.ack(content.clone());
        let msg = Message::Bcast(content);
        let msg = serde_json::to_string(&msg).unwrap();
        for other in self.others() {
            send_message(&other, msg.clone());
        }
    }

    fn hash(&self) -> model::system::HashType {
        let mut hasher = DefaultHasher::new();
        let mut pending = self.pending.iter().collect::<Vec<_>>();
        pending.sort();
        for (msg, acks) in pending.into_iter() {
            msg.hash(&mut hasher);
            acks.min(&self.proc_cnt.div_ceil(2)).hash(&mut hasher);
        }
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn build(s: model::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = Bcast::new(nodes, proc);
        let mut node = model::node::Node::new(node_name);
        node.add_proc("bcast", proc).unwrap();
        s.add_node_with_role(node, "bcast").unwrap();
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

#[should_panic]
#[test]
fn one_message_udp_drop_bfs() {
    let log = one_msg::udp_drops_bfs(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn one_message_udp_drop_dfs() {
    let log = one_msg::udp_drops_dfs(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_concurrent_messages_with_faults() {
    let log = two_msg::concurrent_with_faults_check_validity_and_agreement(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_concurrent_messages_without_faults() {
    let log = two_msg::concurrent_without_faults_check_validity_and_agreement(build).unwrap();
    println!("{}", log);
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn two_sequenced_messages_without_faults() {
    let log = two_msg::send_after_recv_no_drop_no_fault_check_all(build).unwrap();
    println!("{log}");
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn two_sequenced_messages_with_faults() {
    let log = two_msg::send_after_recv_no_drop_with_fault_check_all(build).unwrap();
    println!("{log}");
}
