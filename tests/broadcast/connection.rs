use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
    time::Duration,
};

////////////////////////////////////////////////////////////////////////////////

async fn connect_durable(to: mc::Address) -> mc::TcpStream {
    loop {
        if let Ok(stream) = mc::TcpStream::connect(&to).await {
            return stream;
        }
        mc::sleep(Duration::from_millis(500)).await;
    }
}

////////////////////////////////////////////////////////////////////////////////

async fn listen_to_durable(to: mc::Address) -> mc::TcpStream {
    loop {
        if let Ok(stream) = mc::TcpListener::listen_to(&to).await {
            return stream;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub async fn connect(to: mc::Address) -> mc::TcpStream {
    tokio::select! {
        stream = connect_durable(to.clone()) => stream,
        stream = listen_to_durable(to.clone()) => stream,
    }
}

////////////////////////////////////////////////////////////////////////////////

struct State {
    con: HashMap<mc::Address, mc::TcpStream>,
    proc: Vec<mc::Address>,
    me: usize,
}

pub struct Connections(Rc<RefCell<State>>);

impl Connections {
    pub fn new(proc: Vec<mc::Address>, me: usize) -> Self {
        let state = State {
            con: Default::default(),
            proc,
            me,
        };
        Self(Rc::new(RefCell::new(state)))
    }

    pub fn make_connections(&self) {
        for i in 0..self.0.borrow().proc.len() {
            if i != self.0.borrow().me {
                let to = self.0.borrow().proc[i].clone();
                self.make_connection(to);
            }
        }
    }

    pub fn make_connection(&self, to: mc::Address) {
        let state = self.0.clone();
        mc::spawn(async move {
            let stream = connect(to.clone()).await;
            let ex = state.borrow_mut().con.insert(to, stream);
            assert!(ex.is_none());
        });
    }
}

impl Hash for Connections {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys = self.0.borrow().con.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for addr in keys {
            addr.hash(state);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Broadcast {
    con: Connections,
}

impl Broadcast {
    fn new(proc: Vec<mc::Address>, me: usize) -> Self {
        let con = Connections::new(proc, me);
        Self { con }
    }
}

impl mc::Process for Broadcast {
    fn on_message(&mut self, _from: mc::Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        if content == "connect" {
            self.con.make_connections();
        }
    }

    fn hash(&self) -> mc::HashType {
        let mut hasher = DefaultHasher::new();
        self.con.hash(&mut hasher);
        hasher.finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

fn build(s: mc::SystemHandle, nodes: usize) {
    let addrs = (0..nodes)
        .map(|node| format!("{node}:proc").into())
        .collect::<Vec<_>>();
    for node in 0..nodes {
        let bcast = Broadcast::new(addrs.clone(), node);
        let mut node = mc::Node::new(node.to_string());
        node.add_proc("proc", bcast).unwrap();
        s.add_node(node).unwrap();
    }
    s.network()
        .set_delays(Duration::from_millis(100), Duration::from_millis(200))
        .unwrap();
    (0..nodes).for_each(|node| {
        s.send_local(&format!("{node}:proc").into(), "connect")
            .unwrap()
    });
}

////////////////////////////////////////////////////////////////////////////////

fn made_connections(s: mc::StateView) -> bool {
    s.system().pending_events() == 0
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn establish_connections_bfs() {
    let nodes = 3;
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::BfsSearcher::new(cfg);
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let log = checker
        .collect(|_| Ok(()), |_| false, made_connections, searcher)
        .unwrap();
    println!("{}", log);
    println!("collected={}", checker.states_count());
    checker.for_each(|s| println!("{}", s.log()));
    assert_eq!(checker.states_count(), 1);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn establish_connections_dfs() {
    let nodes = 3;
    let cfg = mc::SearchConfig::no_faults_no_drops();
    let searcher = mc::DfsSearcher::new(cfg);
    let mut checker = mc::ModelChecker::new_with_build(move |s| build(s, nodes));
    let log = checker
        .collect(|_| Ok(()), |_| false, made_connections, searcher)
        .unwrap();
    println!("{}", log);
    println!("collected={}", checker.states_count());
    checker.for_each(|s| println!("{}", s.log()));
    assert_eq!(checker.states_count(), 1);
}
