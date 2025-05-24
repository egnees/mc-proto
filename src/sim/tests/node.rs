use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    rpc, send_local, spawn, Address, HashType, Node, Process, RpcListener, Simulation, StepConfig,
};

use crate::fs::file::File;

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct EchoRequest {
    message: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct EchoResponse {
    message: String,
}

////////////////////////////////////////////////////////////////////////////////

pub struct EchoServer {}

impl Process for EchoServer {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, _content: String) {
        // init
        spawn(async {
            let mut file = if let Ok(file) = File::open("file.txt") {
                send_local("open");
                file
            } else {
                let mut file = File::create("file.txt").unwrap();
                send_local("create");
                file.write("hello".as_bytes(), 0).await.unwrap();
                file
            };
            let mut buf = [0u8; 10];
            file.read(&mut buf, 0).await.unwrap();
            assert_eq!(&buf[..5], "hello".as_bytes());
        });

        let mut listener = RpcListener::register().unwrap();
        spawn(async move {
            loop {
                let request = listener.listen().await;
                let content: EchoRequest = request.unpack().unwrap();
                let message = content.message;
                let response = EchoResponse { message };
                request.reply(&response).unwrap();
            }
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct EchoClient {
    server: Address,
}

impl Process for EchoClient {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, message: String) {
        let request = EchoRequest { message };
        let to = self.server.clone();
        spawn(async move {
            let result = rpc(to, 0, &request).await;
            if let Ok(response) = result {
                let message = response.unpack::<EchoResponse>().unwrap().message;
                assert_eq!(message, request.message);
                send_local(message);
            } else {
                send_local("fail");
            }
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn restart() {
    let sim = Simulation::new(123);
    let s = sim.system();
    let server = Node::new("server");
    s.add_node(server).unwrap();
    s.setup_fs(
        "server",
        Duration::from_millis(1),
        Duration::from_millis(3),
        4096,
    )
    .unwrap();
    let server = s.add_proc_on_node("server", "proc", EchoServer {}).unwrap();
    let client = Node::new("client");
    s.add_node(client).unwrap();
    let client = s
        .add_proc_on_node(
            "client",
            "proc",
            EchoClient {
                server: server.address(),
            },
        )
        .unwrap();
    s.send_local(&server.address(), "init").unwrap();
    let cfg = StepConfig {
        udp_packet_drop_prob: 0.0,
    };
    sim.step_until_no_events(&cfg);
    let locals = s
        .read_locals(server.address().node, server.address().process)
        .unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "create");

    s.send_local(&client.address(), "hello").unwrap();
    sim.step_until_no_events(&cfg);
    let locals = s
        .read_locals(client.address().node, client.address().process)
        .unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "hello");

    s.shutdown_node("server").unwrap();
    sim.step_until_no_events(&cfg);

    s.send_local(&client.address(), "hello").unwrap();
    sim.step_until_no_events(&cfg);
    let locals = s
        .read_locals(client.address().node, client.address().process)
        .unwrap();
    assert_eq!(locals.len(), 2);
    assert_eq!(locals[1], "fail");

    s.restart_node("server").unwrap();
    let server = s.add_proc_on_node("server", "proc", EchoServer {}).unwrap();
    s.send_local(&server.address(), "init").unwrap();
    sim.step_until_no_events(&cfg);
    let locals = s
        .read_locals(server.address().node, server.address().process)
        .unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "open");

    s.send_local(&client.address(), "hello2").unwrap();
    sim.step_until_no_events(&cfg);
    let locals = s
        .read_locals(client.address().node, client.address().process)
        .unwrap();
    assert_eq!(locals.len(), 3);
    assert_eq!(locals[2], "hello2");
}
