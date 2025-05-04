use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    event::driver::EventDriver,
    search::{gen::Generator, state::SearchState, step::StateTraceStep},
    send_local, sleep, spawn, Address, HashType, NetConfig, Node, Process, SearchConfig, System,
    SystemHandle, TcpError, TcpListener, TcpStream,
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
            let bytes = stream.send("hello".as_bytes()).await.unwrap();
            assert_eq!(bytes, "hello".len());
            let mut buf = [0u8; 10];
            let bytes = stream.recv(&mut buf).await.unwrap();
            assert_eq!(&buf[..bytes], "hello".as_bytes());
            let recv_result = stream.recv(&mut buf).await;
            assert!(recv_result.is_err());
            assert_eq!(recv_result.err().unwrap(), TcpError::ConnectionRefused);
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

#[test]
fn collect() {
    // build system
    let gen = Rc::new(RefCell::new(Generator::new()));
    let net = NetConfig::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap();
    let system = System::new(&net, &(gen.clone() as Rc<RefCell<dyn EventDriver>>));
    build_system(system.handle());
    let mut state = SearchState { system, gen };

    // apply first step (timer)
    let cfg = SearchConfig::no_faults_no_drops();
    let steps = state.gen.borrow().steps(state.system.handle(), &cfg);
    println!("{:?}", steps);
    assert_eq!(steps.len(), 1);
    assert!(matches!(&steps[0], StateTraceStep::SelectTcpEvent(_, _)));
    steps[0].apply(&mut state).unwrap();

    let steps = state.gen.borrow().steps(state.system.handle(), &cfg);
    println!("{:?}", steps);
    assert_eq!(steps.len(), 1);
    assert!(matches!(&steps[0], StateTraceStep::SelectTimer(_, _)));
    steps[0].apply(&mut state).unwrap();

    // apply second step (tcp)
    let steps = state.gen.borrow().steps(state.system.handle(), &cfg);
    assert_eq!(steps.len(), 1);
    println!("{:?}", steps);
    assert!(matches!(&steps[0], StateTraceStep::SelectTcpPacket(_, _)));
    steps[0].apply(&mut state).unwrap();

    // apply third step
    let steps = state.gen.borrow().steps(state.system.handle(), &cfg);

    // check only one tcp msg appear
    assert_eq!(steps.len(), 1);
    assert!(matches!(&steps[0], StateTraceStep::SelectTcpPacket(_, _)));
}
