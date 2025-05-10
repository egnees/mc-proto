use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    rpc, send_local, spawn, Address, HashType, Node, Process, RpcListener, Simulation, StepConfig,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct IncRequest {
    on: u64,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct IncResult {
    current: u64,
}

////////////////////////////////////////////////////////////////////////////////

pub struct RpcClient {
    to: Address,
    cur_value: u64,
}

impl Process for RpcClient {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let x: IncRequest = serde_json::from_str(content.as_str()).unwrap();
        self.cur_value += x.on;
        let expect = self.cur_value;
        let to = self.to.clone();
        spawn(async move {
            let response = rpc(to, 0, &x).await.unwrap();
            let result: IncResult = response.unpack().unwrap();
            assert_eq!(result.current, expect);
            send_local(serde_json::to_string(&result).unwrap());
        });
    }

    fn hash(&self) -> HashType {
        self.cur_value
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct IncServer {
    value: Rc<RefCell<u64>>,
    requests: usize,
}

impl Process for IncServer {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        assert_eq!(content, "listen");
        let mut listener = RpcListener::register().unwrap();
        let value = self.value.clone();
        let req = self.requests;
        spawn(async move {
            for _ in 0..req {
                let request = listener.listen().await;
                let inc: IncRequest = request.unpack().unwrap();
                *value.borrow_mut() += inc.on;
                let result = IncResult {
                    current: *value.borrow(),
                };
                request.reply(&result).unwrap();
            }
        });
    }

    fn hash(&self) -> HashType {
        *self.value.borrow()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic() {
    let sim = Simulation::new(12345);
    let system = sim.system();

    let mut server = Node::new("s");
    let server_addr = server
        .add_proc(
            "s",
            IncServer {
                value: Rc::new(RefCell::new(0)),
                requests: 1,
            },
        )
        .unwrap()
        .address();
    system.add_node(server).unwrap();

    let mut client = Node::new("c");
    let client_addr = client
        .add_proc(
            "c",
            RpcClient {
                to: server_addr.clone(),
                cur_value: 0,
            },
        )
        .unwrap()
        .address();
    system.add_node(client).unwrap();

    system.send_local(&server_addr, "listen").unwrap();

    let req = IncRequest { on: 2 };
    system
        .send_local(&client_addr, serde_json::to_string(&req).unwrap())
        .unwrap();

    sim.step_until_no_events(&StepConfig::no_drops());

    let locals = system.read_locals("c", "c").unwrap();
    assert_eq!(locals.len(), 1);
    let msg = &locals[0];
    let result: IncResult = serde_json::from_str(msg.as_str()).unwrap();
    assert_eq!(result.current, 2);

    println!("{}", system.log());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic2() {
    for seed in 0..100 {
        let sim = Simulation::new(seed);
        let system = sim.system();

        let mut server = Node::new("s");
        let server_addr = server
            .add_proc(
                "s",
                IncServer {
                    value: Rc::new(RefCell::new(0)),
                    requests: 2,
                },
            )
            .unwrap()
            .address();
        system.add_node(server).unwrap();

        let mut client = Node::new("c");
        let client_addr = client
            .add_proc(
                "c",
                RpcClient {
                    to: server_addr.clone(),
                    cur_value: 0,
                },
            )
            .unwrap()
            .address();
        system.add_node(client).unwrap();

        system.send_local(&server_addr, "listen").unwrap();

        let req = IncRequest { on: 2 };
        system
            .send_local(&client_addr, serde_json::to_string(&req).unwrap())
            .unwrap();

        let req = IncRequest { on: 3 };
        system
            .send_local(&client_addr, serde_json::to_string(&req).unwrap())
            .unwrap();

        sim.step_until_no_events(&StepConfig::no_drops());

        let locals = system.read_locals("c", "c").unwrap();
        assert_eq!(locals.len(), 2);
        let msg = &locals[0];
        let result: IncResult = serde_json::from_str(msg.as_str()).unwrap();
        assert_eq!(result.current, 2);

        let msg = &locals[1];
        let result: IncResult = serde_json::from_str(msg.as_str()).unwrap();
        assert_eq!(result.current, 5);

        if seed == 5 {
            println!("{}", system.log());
        }
    }
}
