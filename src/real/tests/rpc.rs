use std::{net::IpAddr, time::Duration};

use crate::{
    model::HashType,
    real::{node::RealNode, rpc, RouteConfig, RouteConfigBuilder},
    send_local, sleep, spawn, Address, Process, RpcListener,
};

////////////////////////////////////////////////////////////////////////////////

pub struct EchoServer {}

impl Process for EchoServer {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        assert_eq!(content, "init");
        let mut listener = RpcListener::register().unwrap();
        spawn(async move {
            loop {
                let request = listener.listen().await;
                spawn(async move {
                    let content = request.unpack::<String>().unwrap();
                    request.reply(&content).unwrap();
                });
            }
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct EchoClient {
    to: Address,
}

impl Process for EchoClient {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let to = self.to.clone();
        spawn(async move {
            let result = rpc(to, 0, &content).await.unwrap();
            let result = result.unpack::<String>().unwrap();
            assert_eq!(result, content);
            send_local(result);
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn run_echo_server(cfg: RouteConfig) {
    let mut node = RealNode::new("server", 123, cfg, String::default());
    let (sender, _receiver) = node.add_proc("echo", EchoServer {}).unwrap();
    sender.send("init");
    node.block_on(futures::future::pending::<()>());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic() {
    let cfg = RouteConfigBuilder::new()
        .add(
            "server:echo",
            ("127.0.0.1".parse::<IpAddr>().unwrap(), 10091),
        )
        .add(
            "client:echo",
            ("127.0.0.1".parse::<IpAddr>().unwrap(), 10092),
        )
        .build();
    let _server_handle = std::thread::spawn({
        let cfg = cfg.clone();
        || run_echo_server(cfg)
    });

    let mut client_node = RealNode::new("client", 123, cfg, String::default());
    let (sender, mut receiver) = client_node
        .add_proc(
            "echo",
            EchoClient {
                to: "server:echo".into(),
            },
        )
        .unwrap();
    client_node.block_on(async move {
        sleep(Duration::from_millis(100)).await;

        sender.send("123");
        let result = receiver.recv::<String>().await.unwrap();
        assert_eq!(result, "123");

        sender.send("321");
        let result = receiver.recv::<String>().await.unwrap();
        assert_eq!(result, "321");
    });
}
