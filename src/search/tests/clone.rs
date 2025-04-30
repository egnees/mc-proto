use crate::search::state::SearchState;
use std::time::Duration;

use generic_clone::view::View;

use crate::{
    search::step::StateTraceStep, send_local, sleep, spawn, time, Address, HashType, Node, Process,
    SearchConfig, SystemHandle, TcpError, TcpListener, TcpStream,
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
            assert!(time1.from < time2.from);
            let mut buf = [0u8; 10];
            let bytes = stream.recv(&mut buf).await.unwrap();
            assert_eq!(&buf[..bytes], "hello".as_bytes());
            let recv_result = stream.recv(&mut buf).await;
            assert!(recv_result.is_err());
            assert_eq!(recv_result.err().unwrap(), TcpError::ConnectionRefused);
            assert!(time1.shift(Duration::from_millis(100)).from <= time().from);
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
fn clone_works() {
    let store = generic_clone::store::Store::new(16000, 3).unwrap();
    let mut view: View<SearchState> = store.allocate().unwrap();
    view.enter(|v| build_system(v.system.handle()));
    let mut view1 = view.clone();
    let cfg = SearchConfig::no_faults_no_drops();
    view.enter(|v| {
        let steps = v.gen.borrow().steps(v.system.handle(), &cfg);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], StateTraceStep::SelectTimer(_, _)));
        steps[0].apply(v).unwrap();
        let steps = v.gen.borrow().steps(v.system.handle(), &cfg);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], StateTraceStep::SelectTcp(_, _)));
        steps[0].apply(v).unwrap();
    });
    view1.enter(|v| {
        let steps = v.gen.borrow().steps(v.system.handle(), &cfg);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], StateTraceStep::SelectTimer(_, _)));
        steps[0].apply(v).unwrap();
        let steps = v.gen.borrow().steps(v.system.handle(), &cfg);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], StateTraceStep::SelectTcp(_, _)));
        steps[0].apply(v).unwrap();
    });
}
