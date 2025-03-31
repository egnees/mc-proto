use std::{
    cell::RefCell,
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
    time::Duration,
};

////////////////////////////////////////////////////////////////////////////////

struct Ping {
    other: mc::Address,
    state: Rc<RefCell<PingState>>,
}

#[derive(PartialEq, Eq, Default)]
struct PingState {
    waiting: BTreeSet<String>,
}

impl PingState {
    fn hash(&self) -> mc::HashType {
        let mut hasher = DefaultHasher::new();
        for w in self.waiting.iter() {
            w.hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl Ping {
    pub fn new(other: mc::Address) -> Self {
        Self {
            other,
            state: Rc::new(RefCell::new(Default::default())),
        }
    }
}

impl mc::Process for Ping {
    fn on_message(&mut self, from: mc::Address, content: String) {
        assert_eq!(from, self.other);
        self.state.borrow_mut().waiting.remove(&content);
        mc::send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        self.state.borrow_mut().waiting.insert(content.clone());
        mc::spawn({
            let state = self.state.clone();
            let receiver = self.other.clone();
            async move {
                while state.borrow().waiting.contains(&content) {
                    mc::send_message(&receiver, content.clone());
                    mc::sleep(Duration::from_secs(1)).await;
                }
            }
        });
    }

    fn hash(&self) -> mc::HashType {
        self.state.borrow().hash()
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Pong {}

impl Pong {
    fn new() -> Self {
        Self {}
    }
}

impl mc::Process for Pong {
    fn on_message(&mut self, from: mc::Address, content: String) {
        mc::send_message(&from, content.clone());
        mc::send_local(content);
    }

    fn on_local_message(&mut self, _content: String) {
        unreachable!()
    }

    fn hash(&self) -> mc::HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build(min_packet_delay: Duration, max_packet_delay: Duration, locals: usize) -> mc::System {
    // configure network
    let net_cfg = mc::NetConfig::new(min_packet_delay, max_packet_delay).unwrap();

    // build sys with network
    let mut sys = mc::System::new(&net_cfg);

    // configure first node
    let mut n1 = mc::Node::new();
    n1.add_proc("ping", Ping::new(mc::Address::new("n2", "pong")))
        .unwrap();
    sys.add_node("n1", n1).unwrap();

    // configure second node
    let mut n2 = mc::Node::new();
    n2.add_proc("pong", Pong::new()).unwrap();
    sys.add_node("n2", n2).unwrap();

    // send local messages to initiate requests
    for i in 0..locals {
        sys.send_local(&mc::Address::new("n1", "ping"), i.to_string())
            .unwrap();
    }

    // builded sys
    sys
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_local() {
    let invariant = |_| Ok(());

    let prune = |_| false;

    let goal = |s: mc::StateHandle| {
        !s.read_locals(&mc::Address::new("n1", "ping"))
            .unwrap()
            .is_empty()
            && !s
                .read_locals(&mc::Address::new("n2", "pong"))
                .unwrap()
                .is_empty()
    };

    let cfg = mc::SearchConfigBuilder::no_faults()
        .max_msg_drops(100)
        .build();
    let searcher = mc::BfsSearcher::new(cfg);

    let checker = mc::ModelChecker::new(|| {
        build(
            Duration::from_secs_f64(0.1),
            Duration::from_secs_f64(0.2),
            1,
        )
    });
    let checked = checker.check(invariant, prune, goal, searcher).unwrap();
    assert!(checked > 0);

    println!("checked={checked}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_locals_bfs() {
    let invariant = |s: mc::StateHandle| {
        let ping_locals = s.read_locals(&mc::Address::new("n1", "ping")).unwrap();
        let pong_locals = s.read_locals(&mc::Address::new("n2", "pong")).unwrap();
        if ping_locals.len() > 2 || pong_locals.len() > 2 {
            Err("too many locals".to_string())
        } else {
            Ok(())
        }
    };

    let prune = |_| false;

    let goal = |s: mc::StateHandle| {
        let ping_locals = s.read_locals(&mc::Address::new("n1", "ping")).unwrap();
        let pong_locals = s.read_locals(&mc::Address::new("n2", "pong")).unwrap();
        ping_locals.len() == 2 && pong_locals.len() == 2
    };

    let cfg = mc::SearchConfigBuilder::no_faults()
        .max_msg_drops(1)
        .build();
    let searcher = mc::BfsSearcher::new(cfg);

    let checker =
        mc::ModelChecker::new(|| build(Duration::from_millis(100), Duration::from_millis(600), 2));
    let result = checker.check(invariant, prune, goal, searcher);
    assert!(result.is_err());

    let err = result.err().unwrap();
    println!("{}", err);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn two_locals_dfs() {
    let invariant = |s: mc::StateHandle| {
        let ping_locals = s.read_locals(&mc::Address::new("n1", "ping")).unwrap();
        let pong_locals = s.read_locals(&mc::Address::new("n2", "pong")).unwrap();
        if ping_locals.len() > 2 || pong_locals.len() > 2 {
            Err("too many locals".to_string())
        } else {
            Ok(())
        }
    };

    let prune = |_| false;

    let goal = |s: mc::StateHandle| {
        let ping_locals = s.read_locals(&mc::Address::new("n1", "ping")).unwrap();
        let pong_locals = s.read_locals(&mc::Address::new("n2", "pong")).unwrap();
        ping_locals.len() == 2 && pong_locals.len() == 2
    };

    let cfg = mc::SearchConfigBuilder::no_faults()
        .max_msg_drops(1)
        .build();
    let searcher = mc::DfsSearcher::new(cfg);

    let checker =
        mc::ModelChecker::new(|| build(Duration::from_millis(100), Duration::from_millis(600), 2));
    let result = checker.check(invariant, prune, goal, searcher);
    assert!(result.is_err());

    let err = result.err().unwrap();
    println!("{}", err);
}
