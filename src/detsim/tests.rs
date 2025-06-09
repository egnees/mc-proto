use std::time::Duration;

use crate::{
    detsim::Simulation, detsim::StepConfig,
    model::node::Node,
};

use serde::{Deserialize, Serialize};

use crate::{
    model::net::send_message, send_local, sleep, spawn,
    Address, HashType, Process,
};

use crate::model::fs::file::File;

////////////////////////////////////////////////////////////////////////////////

pub struct Pinger {
    pub receiver: Address,
}

impl Process for Pinger {
    fn on_message(
        &mut self,
        from: Address,
        content: String,
    ) {
        assert_eq!(from, self.receiver);
        send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        send_message(&self.receiver, content);
    }

    fn hash(&self) -> HashType {
        unreachable!()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Ponger {}

impl Process for Ponger {
    fn on_message(
        &mut self,
        from: Address,
        content: String,
    ) {
        send_message(&from, content.clone());
        send_local(content);
    }

    fn on_local_message(&mut self, _content: String) {
        unreachable!()
    }

    fn hash(&self) -> HashType {
        unreachable!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Sleeper {}

impl Process for Sleeper {
    fn on_message(
        &mut self,
        _from: Address,
        _content: String,
    ) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let ms = u64::from_str_radix(content.as_str(), 10)
            .unwrap();
        spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            send_local(content);
        });
    }

    fn hash(&self) -> HashType {
        unreachable!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub enum Msg {
    CreateFile(String),
    DeleteFile(String),
    Read {
        file: String,
        offset: usize,
        len: usize,
    },
    Write {
        file: String,
        offset: usize,
        content: String,
    },
}

impl From<String> for Msg {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap()
    }
}

impl From<Msg> for String {
    fn from(value: Msg) -> Self {
        serde_json::to_string(&value).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Store {}

impl Process for Store {
    fn on_message(
        &mut self,
        _from: Address,
        _content: String,
    ) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let msg: Msg = content.into();
        match msg {
            Msg::CreateFile(file) => {
                File::create(file).unwrap();
            }
            Msg::DeleteFile(file) => {
                File::delete(file).unwrap();
            }
            Msg::Read { file, offset, len } => {
                spawn(async move {
                    let mut file =
                        File::open(file).unwrap();
                    let mut v = vec![0; len];
                    let bytes = file
                        .read(v.as_mut_slice(), offset)
                        .await
                        .unwrap();
                    let result = String::from_iter(
                        v.as_slice()[..bytes]
                            .iter()
                            .map(|u| char::from(*u)),
                    );
                    send_local(result);
                });
            }
            Msg::Write {
                file,
                offset,
                content,
            } => {
                spawn(async move {
                    let mut file =
                        File::open(file).unwrap();
                    file.write(content.as_bytes(), offset)
                        .await
                        .unwrap();
                });
            }
        };
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build_sim() -> Simulation {
    let sim = Simulation::new(123);
    let mut node = Node::new("n1");
    node.add_proc("p1", Store::default()).unwrap();
    sim.system().add_node(node).unwrap();
    sim.system()
        .setup_fs(
            "n1",
            Duration::from_millis(20),
            Duration::from_millis(100),
            100,
        )
        .unwrap();
    sim
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic_fs() {
    let cfg = StepConfig::new(0.);

    let sim = build_sim();

    sim.system()
        .send_local(
            &"n1:p1".into(),
            Msg::CreateFile("f1".into()),
        )
        .unwrap();

    sim.step_until_no_events(&cfg);

    sim.system()
        .send_local(
            &"n1:p1".into(),
            Msg::Write {
                file: "f1".into(),
                offset: 0,
                content: "hello".into(),
            },
        )
        .unwrap();

    sim.step_until_no_events(&cfg);

    sim.system()
        .send_local(
            &"n1:p1".into(),
            Msg::Read {
                file: "f1".into(),
                offset: 0,
                len: 5,
            },
        )
        .unwrap();

    sim.step_until_no_events(&cfg);

    let locals =
        sim.system().read_locals("n1", "p1").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "hello");
}
