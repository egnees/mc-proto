use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc, time::Duration};

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

async fn connect(to: mc::Address) -> mc::TcpStream {
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
