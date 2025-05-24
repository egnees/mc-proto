use std::{cell::RefCell, rc::Rc, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    event::driver::EventDriver,
    search::{gen::Generator, state::SearchState, step::StateTraceStep},
    send_local, spawn, Address, HashType, Node, Process, SearchConfig, System, SystemHandle,
};

use crate::fs::file::File;

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
struct Store {}

impl Process for Store {
    fn on_message(&mut self, _from: Address, _content: String) {
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
                    let mut file = File::open(file).unwrap();
                    let mut v = vec![0; len];
                    let bytes = file.read(v.as_mut_slice(), offset).await.unwrap();
                    let result =
                        String::from_iter(v.as_slice()[..bytes].iter().map(|u| char::from(*u)));
                    send_local(result);
                });
            }
            Msg::Write {
                file,
                offset,
                content,
            } => {
                spawn(async move {
                    let mut file = File::open(file).unwrap();
                    file.write(content.as_bytes(), offset).await.unwrap();
                });
            }
        };
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build_system(s: SystemHandle) {
    let mut node = Node::new("n");
    node.add_proc("p", Store {}).unwrap();
    s.add_node(node).unwrap();
    s.setup_fs(
        "n",
        Duration::from_millis(20),
        Duration::from_millis(100),
        100,
    )
    .unwrap();
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic() {
    // build system
    let gen = Rc::new(RefCell::new(Generator::new()));
    let system = System::new_default_net(&(gen.clone() as Rc<RefCell<dyn EventDriver>>));
    build_system(system.handle());
    let mut state = SearchState { system, gen };

    let cfg = SearchConfig::no_faults_no_drops();

    // create file f1
    state
        .system
        .handle()
        .send_local(&"n:p".into(), Msg::CreateFile("f1".into()))
        .unwrap();

    let steps = state.steps(&cfg);
    assert!(steps.is_empty());

    // write to f1
    state
        .system
        .handle()
        .send_local(
            &"n:p".into(),
            Msg::Write {
                file: "f1".into(),
                offset: 0,
                content: "hello\n".into(),
            },
        )
        .unwrap();

    // check async operation produced step
    let steps = state.steps(&cfg);
    assert!(!steps.is_empty());
    assert!(matches!(steps[0], StateTraceStep::SelectFsEvent(..)));

    // select write
    steps[0].apply(&mut state).unwrap();

    // make two read requests

    // first
    state
        .system
        .handle()
        .send_local(
            &"n:p".into(),
            Msg::Read {
                file: "f1".into(),
                offset: 0,
                len: 5,
            },
        )
        .unwrap();

    // second
    state
        .system
        .handle()
        .send_local(
            &"n:p".into(),
            Msg::Read {
                file: "f1".into(),
                offset: 1,
                len: 5,
            },
        )
        .unwrap();

    let steps = state.steps(&cfg);
    assert_eq!(steps.len(), 1);

    steps[0].apply(&mut state).unwrap();
    let msgs = state.system.handle().read_locals("n", "p").unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0], "hello");

    let steps = state.steps(&cfg);
    assert_eq!(steps.len(), 1);

    steps[0].apply(&mut state).unwrap();
    let msgs = state.system.handle().read_locals("n", "p").unwrap();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[1], "ello\n");
}
