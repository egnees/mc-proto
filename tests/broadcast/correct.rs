use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use super::connection::Connections;

////////////////////////////////////////////////////////////////////////////////

pub struct Broadcast {
    con: Connections,
}

impl Broadcast {
    fn new(proc: Vec<mc::Address>, me: usize) -> Self {
        let con = Connections::new(proc, me);
        Self { con }
    }
}

impl mc::Process for Broadcast {
    fn on_message(&mut self, _from: mc::Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        if content == "connect" {
            self.con.make_connections();
        }
    }

    fn hash(&self) -> mc::HashType {
        let mut hasher = DefaultHasher::new();
        self.con.hash(&mut hasher);
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build(s: mc::SystemHandle, nodes: usize) {
    let addrs = (0..nodes)
        .map(|node| format!("{node}:proc").into())
        .collect::<Vec<_>>();
    for node in 0..nodes {
        let bcast = Broadcast::new(addrs.clone(), node);
        let mut node = mc::Node::new(node.to_string());
        node.add_proc("proc", bcast).unwrap();
        s.add_node(node).unwrap();
    }
    s.network()
        .set_delays(Duration::from_millis(100), Duration::from_millis(200))
        .unwrap();
    (0..nodes).for_each(|node| {
        s.send_local(&format!("{node}:proc").into(), "connect")
            .unwrap()
    });
}

////////////////////////////////////////////////////////////////////////////////

fn made_connections(s: mc::SystemHandle) -> bool {
    s.pending_events() == 0
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn establish_connections() {
    let nodes = 3;
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let collected = checker
        .collect(|_| Ok(()), |_| false, made_connections, searcher)
        .unwrap();
    println!("collected = '{collected}'");
    checker.for_each(|s| println!("{}", s.log()));
}
