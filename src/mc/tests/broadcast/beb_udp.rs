use std::hash::{DefaultHasher, Hash, Hasher};

use super::two_msg;

use super::one_msg;

use crate::model::{self, net::send_message};

pub use crate::prelude::*;

////////////////////////////////////////////////////////////////////////////////
/// Best Effort Broadcast
////////////////////////////////////////////////////////////////////////////////

struct BebProcess {
    others: Vec<Address>,
    me: usize,
    reg: Vec<String>,
}

impl BebProcess {
    fn new(others: usize, me: usize) -> Self {
        Self {
            others: (0..others)
                .map(|n| format!("{n}:bcast").into())
                .collect::<Vec<_>>(),
            me,
            reg: Default::default(),
        }
    }
}

impl Process for BebProcess {
    fn on_message(&mut self, _from: Address, content: String) {
        self.reg.push(content.clone());
        send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        self.reg.push(content.clone());
        for i in 0..self.others.len() {
            if i != self.me {
                send_message(&self.others[i], &content);
            }
        }
        send_local(content);
    }

    fn hash(&self) -> model::system::HashType {
        let mut hasher = DefaultHasher::new();
        self.reg.iter().for_each(|s| s.hash(&mut hasher));
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn build(s: model::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = BebProcess::new(nodes, proc);
        let mut node = model::node::Node::new(node_name);
        node.add_proc("bcast", proc).unwrap();
        s.add_node(node).unwrap();
    });
}

pub fn build_with_roles(s: model::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc = BebProcess::new(nodes, proc);
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
fn one_message_no_faults_with_role() {
    let log_roles = one_msg::no_drops(build_with_roles).unwrap();
    println!("{}", log_roles);

    let log_without_roles = one_msg::no_drops(build).unwrap();
    println!("{}", log_without_roles);

    assert!(log_roles.visited_unique < log_without_roles.visited_total);
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
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
