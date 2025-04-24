use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::{connection, one_msg};

////////////////////////////////////////////////////////////////////////////////
/// Best Effort Broadcast
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct StreamParser {
    accum: String,
}

impl StreamParser {
    pub fn parse(&mut self, buf: &[u8]) {
        for c in buf.iter().copied() {
            if c == b'\n' {
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
) -> Result<(), mc::TcpError> {
    let mut stream = connection::connect(with).await;
    let mut buf = [0u8; 1024];
    let mut parser = StreamParser::default();
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
    locals: Vec<String>,
    me: usize,
}

impl BebProcess {
    fn new(others: usize, me: usize) -> Self {
        Self {
            proc: (0..others)
                .map(|n| format!("{n}:{n}").into())
                .collect::<Vec<_>>(),
            senders: Default::default(),
            locals: Default::default(),
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
            self.locals.push(content.clone());
        }

        if self.senders.is_empty() {
            let others = self.iter_others().cloned().collect::<Vec<_>>();
            others.into_iter().for_each(|other| {
                let (sender, receiver) = unbounded_channel();
                self.senders.insert(other.clone(), sender);
                mc::spawn(communicate_with(other, receiver));
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
        let proc_handle = node.add_proc(proc_name, proc).unwrap();
        s.add_node(node).unwrap();
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
fn one_message_no_duplications() {
    let log = one_msg::no_drops_no_faults_check_no_duplications(build).unwrap();
    println!("{}", log);
}
