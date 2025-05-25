use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use crate::{
    mc::error::SearchErrorKind, mc::BfsSearcher, mc::ModelChecker, mc::SearchConfig,
    mc::SearchConfigBuilder, mc::StateView, model::log::LogEntry, model::node::Node,
    model::proc::time, model::tcp::listen::TcpListener, model::tcp::stream::TcpStream,
    model::tcp::TcpError, model::HashType, model::SystemHandle, send_local, sleep, spawn, Address,
    Process,
};

////////////////////////////////////////////////////////////////////////////////

struct Sender {}

impl Sender {
    async fn connect_to(to: Address) -> TcpStream {
        loop {
            if let Ok(stream) = TcpStream::connect(&to).await {
                return stream;
            }
            sleep(Duration::from_millis(500)).await;
        }
    }
}

impl Process for Sender {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let to: Address = content.into();
        spawn(async move {
            let mut stream = Self::connect_to(to).await;
            let time1 = time();
            let bytes = stream.send("hello".as_bytes()).await.unwrap();
            assert_eq!(bytes, "hello".len());
            let time2 = time();
            assert!(time1 < time2);
            let mut buf = [0u8; 10];
            let bytes = stream.recv(&mut buf).await.unwrap();
            assert_eq!(&buf[..bytes], "hello".as_bytes());
            let recv_result = stream.recv(&mut buf).await;
            assert!(recv_result.is_err());
            assert_eq!(recv_result.err().unwrap(), TcpError::ConnectionRefused);
            assert!(time1 + Duration::from_millis(100) <= time());
            send_local("done");
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Receiver {}

impl Process for Receiver {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, _content: String) {
        spawn(async {
            spawn(async {
                let listen_result = TcpListener::listen().await;
                assert!(listen_result.is_err());
            });
            let mut stream = TcpListener::listen().await.unwrap();
            let mut buf = [0u8; 10];
            let bytes = stream.recv(&mut buf).await.unwrap();
            assert_eq!(&buf[..bytes], "hello".as_bytes());
            let bytes = stream.send(&buf[..bytes]).await.unwrap();
            assert_eq!(bytes, "hello".len());
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build_system(sys: SystemHandle) {
    sys.network()
        .set_delays(Duration::from_millis(200), Duration::from_millis(400))
        .unwrap();

    let mut n1 = Node::new("node1");
    let sender = n1.add_proc("sender", Sender {}).unwrap();
    sys.add_node(n1).unwrap();
    sys.send_local(&sender.address(), "node2:recv").unwrap();

    let mut n2 = Node::new("node2");
    let recv = n2.add_proc("recv", Receiver {}).unwrap();
    sys.add_node(n2).unwrap();
    sys.send_local(&recv.address(), "spawn").unwrap();
}

////////////////////////////////////////////////////////////////////////////////

fn goal(sys: StateView) -> Result<(), String> {
    if sys.system().pending_events() == 0
        && sys
            .system()
            .read_locals("node1", "sender")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    {
        Ok(())
    } else {
        Err("error".into())
    }
}

////////////////////////////////////////////////////////////////////////////////

fn invariant(s: StateView) -> Result<(), String> {
    let mut ord: HashMap<(Address, Address), VecDeque<LogEntry>> = HashMap::new();
    let sys = s.system();
    for e in sys.log().iter() {
        if let LogEntry::TcpMessageSent(m) = e {
            ord.entry((m.from.clone(), m.to.clone()))
                .or_default()
                .push_back(e.clone());
        } else if let LogEntry::TcpMessageReceived(m) = e {
            let send = ord
                .get_mut(&(m.from.clone(), m.to.clone()))
                .and_then(|v| v.pop_front())
                .ok_or("tcp violated".to_string())?;
            assert!(matches!(send, LogEntry::TcpMessageSent(_)));
            let send = variant::variant!(send, LogEntry::TcpMessageSent(send));
            if send.packet != m.packet {
                return Err(format!(
                    "packets not correspond, send={:?}, recv={:?}",
                    send.packet, m.packet
                ));
            }
        } else if let LogEntry::TcpMessageDropped(m) = e {
            let send = ord
                .get_mut(&(m.from.clone(), m.to.clone()))
                .and_then(|v| v.pop_front())
                .ok_or("tcp violated".to_string())?;
            assert!(matches!(send, LogEntry::TcpMessageSent(_)));
            let send = variant::variant!(send, LogEntry::TcpMessageSent(send));
            if send.packet != m.packet {
                return Err(format!(
                    "packets not correspond, send={:?}, drop={:?}",
                    send.packet, m.packet
                ));
            }
        }
    }
    if s.depth() >= 20 {
        Err(format!("too big depth: {}", s.depth()))
    } else {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn collect() {
    let mut checker = ModelChecker::new_with_build(build_system);
    let searcher = BfsSearcher::new(SearchConfig::no_faults_no_drops());
    let log = checker
        .collect(invariant, |_| false, goal, searcher)
        .unwrap();
    println!("{}", log);
    assert_eq!(checker.states_count(), 1);
    checker.for_each(|s| println!("{}", s.log()));
    checker.for_each(|s| assert!(!s.read_locals("node1", "sender").unwrap().is_empty()));
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn check_no_faults() {
    let checker = ModelChecker::new_with_build(build_system);
    let searcher = BfsSearcher::new(SearchConfig::no_faults_no_drops());
    let log = checker.check(invariant, |_| false, goal, searcher).unwrap();
    println!("{}", log);
    assert!(log.visited_total > 0);
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn check_node_fault() {
    let checker = ModelChecker::new_with_build(build_system);
    let cfg = SearchConfigBuilder::new()
        .max_node_faults(1)
        .max_msg_drops(0)
        .max_disk_faults(0)
        .build();
    let searcher = BfsSearcher::new(cfg);
    let log = checker.check(invariant, |_| false, goal, searcher).unwrap();
    println!("{}", log);
    assert!(log.visited_total > 0);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn both_fail() {
    let checker = ModelChecker::new_with_build(build_system);
    let searcher = BfsSearcher::new(SearchConfig::with_node_faults_only(2));
    let err = checker.check(invariant, |_| false, goal, searcher);
    assert!(err.is_err());
    assert!(matches!(
        err.unwrap_err().kind,
        SearchErrorKind::LivenessViolation(..),
    ));
}
