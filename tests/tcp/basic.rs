use std::time::Duration;

use mc::{Address, Process};

////////////////////////////////////////////////////////////////////////////////

struct Sender {}

impl Sender {
    async fn connect_to(to: Address) -> mc::TcpStream {
        loop {
            if let Ok(stream) = mc::TcpStream::connect(&to).await {
                return stream;
            }
            mc::sleep(Duration::from_millis(500)).await;
        }
    }
}

impl Process for Sender {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let to: Address = content.into();
        mc::spawn(async move {
            let mut stream = Self::connect_to(to).await;
            let time1 = mc::time();
            let bytes = stream.send("hello".as_bytes()).await.unwrap();
            assert_eq!(bytes, "hello".len());
            let time2 = mc::time();
            assert!(time1.from < time2.from);
            let mut buf = [0u8; 10];
            let bytes = stream.recv(&mut buf).await.unwrap();
            assert_eq!(&buf[..bytes], "hello".as_bytes());
            let recv_result = stream.recv(&mut buf).await;
            assert!(recv_result.is_err());
            assert_eq!(recv_result.err().unwrap(), mc::TcpError::ConnectionRefused);
            assert!(time1.shift(Duration::from_millis(100)).from <= mc::time().from);
            mc::send_local("done");
        });
    }

    fn hash(&self) -> mc::HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Receiver {}

impl mc::Process for Receiver {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, _content: String) {
        mc::spawn(async {
            mc::spawn(async {
                let listen_result = mc::TcpListener::listen().await;
                assert!(listen_result.is_err());
            });
            let mut stream = mc::TcpListener::listen().await.unwrap();
            let mut buf = [0u8; 10];
            let bytes = stream.recv(&mut buf).await.unwrap();
            assert_eq!(&buf[..bytes], "hello".as_bytes());
            let bytes = stream.send(&buf[..bytes]).await.unwrap();
            assert_eq!(bytes, "hello".len());
        });
    }

    fn hash(&self) -> mc::HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build_system(sys: mc::SystemHandle) {
    sys.network()
        .set_delays(Duration::from_millis(200), Duration::from_millis(400))
        .unwrap();

    let mut n1 = mc::Node::new("node1");
    let sender = n1.add_proc("sender", Sender {}).unwrap();
    sys.add_node(n1).unwrap();
    sys.send_local(&sender.address(), "node2:recv").unwrap();

    let mut n2 = mc::Node::new("node2");
    let recv = n2.add_proc("recv", Receiver {}).unwrap();
    sys.add_node(n2).unwrap();
    sys.send_local(&recv.address(), "spawn").unwrap();
}

////////////////////////////////////////////////////////////////////////////////

fn goal(sys: mc::SystemHandle) -> bool {
    sys.pending_events() == 0
}

////////////////////////////////////////////////////////////////////////////////

fn invariant(sys: mc::SystemHandle) -> Result<(), String> {
    let mut tcp_msgs = 0;
    for e in sys.log().iter() {
        if matches!(e, mc::LogEntry::TcpMessageSent(_)) {
            tcp_msgs += 1;
        }
    }
    if tcp_msgs >= 10 {
        Err(format!("too many tcp msgs: {tcp_msgs}"))
    } else {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn establish_connection() {
    let mut checker = mc::ModelChecker::new_with_build(build_system);
    let searcher = mc::BfsSearcher::new(mc::SearchConfig::no_faults_no_drops());
    let collected = checker
        .collect(invariant, |_| false, goal, searcher)
        .unwrap();
    println!("collected = {}", collected);
    assert_eq!(checker.states_count(), 1);
    checker.for_each(|s| println!("{}", s.log()));
    checker.for_each(|s| assert!(!s.read_locals("node1", "sender").unwrap().is_empty()));
}
