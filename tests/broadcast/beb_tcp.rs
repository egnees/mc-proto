use std::hash::{DefaultHasher, Hash, Hasher};

use super::one_msg;

////////////////////////////////////////////////////////////////////////////////
/// Best Effort Broadcast
////////////////////////////////////////////////////////////////////////////////

struct BebProcess {
    others: Vec<mc::Address>,
    me: usize,
    locals: Vec<String>,
}

impl BebProcess {
    fn new(others: usize, me: usize) -> Self {
        Self {
            others: (0..others)
                .map(|n| format!("{n}:{n}").into())
                .collect::<Vec<_>>(),
            me,
            locals: Default::default(),
        }
    }
}

impl mc::Process for BebProcess {
    fn on_message(&mut self, _from: mc::Address, content: String) {
        mc::send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        self.locals.push(content.clone());
        for i in 0..self.others.len() {
            if i != self.me {
                mc::send_message(&self.others[i], &content);
            }
        }
        mc::send_local(content);
    }

    fn hash(&self) -> mc::HashType {
        let mut hasher = DefaultHasher::new();
        self.locals.iter().for_each(|s| s.hash(&mut hasher));
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn build(s: mc::SystemHandle, nodes: usize) {
    (0..nodes).into_iter().for_each(|node| {
        let node_name = node.to_string();
        let proc = node;
        let proc_name = proc.to_string();
        let proc = BebProcess::new(nodes, proc);
        let mut node = mc::Node::new(node_name);
        node.add_proc(proc_name, proc).unwrap();
        s.add_node(node).unwrap();
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
fn one_message_no_duplications() {
    let log = one_msg::no_drops_no_faults_check_no_duplications(build).unwrap();
    println!("{}", log);
}
