use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    event::driver::EventDriver,
    search::{config::SearchConfig, gen::Generator, state::SearchState, step::StateTraceStep},
    NetConfig, Node, System,
};

////////////////////////////////////////////////////////////////////////////////

#[test]
fn node_shutdown() {
    let gen = Rc::new(RefCell::new(Generator::new()));
    let net = NetConfig::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap();

    let system = System::new(&net, &(gen.clone() as Rc<RefCell<dyn EventDriver>>));
    system.handle().add_node(Node::new("node1")).unwrap();
    system.handle().add_node(Node::new("node2")).unwrap();

    let mut state = SearchState { system, gen };

    let steps = state.steps(&SearchConfig::with_node_shutdown_only(2));
    assert_eq!(steps.len(), 2);
    assert!(matches!(steps[0], StateTraceStep::ShutdownNode(0)));
    assert!(matches!(steps[1], StateTraceStep::ShutdownNode(1)));

    steps[0].apply(&mut state).unwrap();

    let available = state.system.handle().node_available_index(0);
    assert!(!available);

    let steps = state.steps(&SearchConfig::with_node_shutdown_only(2));
    assert_eq!(steps.len(), 1);
    assert!(matches!(steps[0], StateTraceStep::ShutdownNode(1)));

    let steps = state.steps(&SearchConfig::with_node_shutdown_only(1));
    assert!(steps.is_empty());
}
