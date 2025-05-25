use std::time::Duration;

use crate::{sim::Simulation, Node, StepConfig};

use super::common::{Msg, Store};

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
        .send_local(&"n1:p1".into(), Msg::CreateFile("f1".into()))
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

    let locals = sim.system().read_locals("n1", "p1").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "hello");
}
