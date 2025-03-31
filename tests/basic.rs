use std::time::Duration;

enum ProcState {
    Init,
    Sent,
    Received,
}

impl ProcState {
    fn hash(&self) -> mc::HashType {
        match self {
            ProcState::Init => 0,
            ProcState::Sent => 1,
            ProcState::Received => 2,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Proc {
    other: mc::Address,
    state: ProcState,
}

impl Proc {
    fn new(other: mc::Address) -> Self {
        Self {
            other,
            state: ProcState::Init,
        }
    }
}

impl mc::Process for Proc {
    fn on_message(&mut self, _from: mc::Address, content: String) {
        self.state = ProcState::Received;
        mc::send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        self.state = ProcState::Sent;
        mc::send_message(&self.other, content);
    }

    fn hash(&self) -> mc::HashType {
        self.state.hash()
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build() -> mc::System {
    // configure network
    let net_cfg =
        mc::NetConfig::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap();

    // build sys with network
    let mut sys = mc::System::new(&net_cfg);

    // configure first node
    let mut n1 = mc::Node::new();
    n1.add_proc("p1", Proc::new(mc::Address::new("n2", "p2")))
        .unwrap();
    sys.add_node("n1", n1).unwrap();

    // configure second node
    let mut n2 = mc::Node::new();
    n2.add_proc("p2", Proc::new(mc::Address::new("n1", "p1")))
        .unwrap();
    sys.add_node("n2", n2).unwrap();

    // send local message to initiate request
    sys.send_local(&mc::Address::new("n1", "p1"), "m").unwrap();

    // builded sys
    sys
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn no_drops() {
    let invariant = |_| Ok(());

    let prune = |_| false;

    let goal = |s: mc::StateHandle| {
        !s.read_locals(&mc::Address::new("n2", "p2"))
            .unwrap()
            .is_empty()
    };

    let searcher = mc::DfsSearcher::new(mc::SearchConfig::no_faults_no_drops());

    let checker = mc::ModelChecker::new(build);
    let checked = checker.check(invariant, prune, goal, searcher).unwrap();

    assert_eq!(checked, 2);
}

#[test]
fn with_drops() {
    let invariant = |_| Ok(());

    let prune = |_| false;

    let goal = |s: mc::StateHandle| {
        !s.read_locals(&mc::Address::new("n2", "p2"))
            .unwrap()
            .is_empty()
    };

    let cfg = mc::SearchConfigBuilder::no_faults()
        .max_msg_drops(1)
        .build();
    let searcher = mc::DfsSearcher::new(cfg);

    let checker = mc::ModelChecker::new(build);
    let result = checker.check(invariant, prune, goal, searcher);
    assert!(result.is_err());
}
