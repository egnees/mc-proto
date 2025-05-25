use std::hash::{DefaultHasher, Hash, Hasher};

use crate::model::{
    event::{info::EventInfo, Event},
    node::Node,
};

pub use crate::{util, Address};

use super::node::NodeRoleRegister;

////////////////////////////////////////////////////////////////////////////////

pub struct HashContext<'a> {
    node_register: &'a NodeRoleRegister,
}

impl<'a> HashContext<'a> {
    pub fn new(node_register: &'a NodeRoleRegister) -> Self {
        Self { node_register }
    }

    fn node_repr(&self, node: &'a str) -> &'a str {
        self.node_register.role(node).unwrap_or(node)
    }

    fn hash_address(&self, a: &Address) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.node_repr(&a.node).hash(&mut hasher);
        a.process.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_node(&self, node: &Node) -> u64 {
        let mut hasher = DefaultHasher::new();
        node.hash(&mut hasher);
        self.node_repr(&node.name).hash(&mut hasher);
        hasher.finish()
    }

    pub fn hash_nodes(&self, nodes: impl Iterator<Item = &'a Node>) -> u64 {
        util::hash::hash_multiset(nodes.map(|n| self.hash_node(n)))
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn hash_event(&self, event: &'a Event) -> u64 {
        let mut hasher = DefaultHasher::new();
        match &event.info {
            EventInfo::UdpMessage(udp) => {
                udp.content.hash(&mut hasher);
                self.hash_address(&udp.from.address()).hash(&mut hasher);
                self.hash_address(&udp.to.address()).hash(&mut hasher);
            }
            EventInfo::TcpMessage(tcp) => {
                tcp.packet.hash(&mut hasher);
                self.hash_address(&tcp.from.address()).hash(&mut hasher);
                self.hash_address(&tcp.to.address()).hash(&mut hasher);
            }
            EventInfo::Timer(timer) => {
                timer.min_duration.hash(&mut hasher);
                timer.max_duration.hash(&mut hasher);
                self.hash_address(&timer.proc.address()).hash(&mut hasher);
            }
            EventInfo::TcpEvent(event) => {
                event.kind.hash(&mut hasher);
                self.hash_address(&event.to.address()).hash(&mut hasher);
            }
            EventInfo::FsEvent(event) => {
                event.kind.hash(&mut hasher);
                self.hash_address(&event.proc).hash(&mut hasher);
            }
            EventInfo::RpcMessage(rpc) => {
                rpc.kind.hash(&mut hasher);
                self.hash_address(&rpc.from.address()).hash(&mut hasher);
                self.hash_address(&rpc.to.address()).hash(&mut hasher);
            }
            EventInfo::RpcEvent(e) => {
                e.kind.hash(&mut hasher);
                self.hash_address(&e.to.address()).hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    pub fn hash_events(&self, events: impl Iterator<Item = &'a Event> + Clone) -> u64 {
        util::hash::hash_multiset(events.map(|e| self.hash_event(e)))
    }
}
