use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    future::Future,
    net::SocketAddr,
    rc::{Rc, Weak},
    time::Duration,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::sync::mpsc::unbounded_channel;

use crate::{util::send::SendSyncWrapper, Address, Process, RpcError, RpcResult};

use super::{
    context::{Context, Guard},
    join::JoinHandle,
    proc::{LocalReceiver, LocalSender, ProcessHandle, ProcessState},
    route::RouteConfig,
    rpc::{self, RpcListener},
    timer::Timer,
    Error,
};

////////////////////////////////////////////////////////////////////////////////

pub struct RealNodeState {
    name: String,
    rt: tokio::runtime::Runtime,
    rng: RefCell<StdRng>,
    net: RouteConfig,
    proc: HashMap<String, Rc<RefCell<ProcessState>>>,
    mount_dir: String,
}

impl RealNodeState {
    fn new(
        name: impl Into<String>,
        seed: u64,
        net: RouteConfig,
        mount_dir: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            rt: tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .unwrap(),
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
            net,
            proc: Default::default(),
            mount_dir: mount_dir.into(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Async activities
    ////////////////////////////////////////////////////////////////////////////////

    fn spawn<F>(&self, f: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send,
    {
        let f = SendSyncWrapper::new(f);
        let task = self.rt.spawn(f);
        JoinHandle { task }
    }

    pub fn block_on<F>(&self, f: F) -> F::Output
    where
        F: Future,
    {
        self.rt.block_on(f)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Timer
    ////////////////////////////////////////////////////////////////////////////////

    fn set_timer(&self, duration: Duration) -> Timer {
        let task = self
            .rt
            .spawn(async move { tokio::time::sleep(duration).await });
        Timer::new(task)
    }

    fn set_random_timer(&self, min_duration: Duration, max_duration: Duration) -> Timer {
        let duration = self
            .rng
            .borrow_mut()
            .random_range(min_duration..=max_duration);
        self.set_timer(duration)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // RPC
    ////////////////////////////////////////////////////////////////////////////////

    fn register_rpc_listener(&self, proc: impl Into<String>) -> RpcResult<RpcListener> {
        let proc_name = proc.into();
        let Some(state) = self.proc.get(&proc_name).cloned() else {
            return Err(RpcError::NotFound);
        };
        if state.borrow().rpc {
            return Err(RpcError::AlreadyListening);
        }
        state.borrow_mut().rpc = true;
        let address = Address::new(&self.name, proc_name);
        let socket_addr = *self.net.get(&address).ok_or(RpcError::NotFound)?;
        let (sender, receiver) = unbounded_channel();
        self.rt.spawn(rpc::server::listen(socket_addr, sender));
        Ok(RpcListener { receiver })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RealNodeHandle(Weak<RefCell<RealNodeState>>);

impl RealNodeHandle {
    pub fn register_rpc_listener(&self, proc: impl Into<String>) -> RpcResult<RpcListener> {
        let s = self.0.upgrade().unwrap();
        let s = s.borrow();
        s.register_rpc_listener(proc)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn mount_dir(&self) -> String {
        self.0.upgrade().unwrap().borrow().mount_dir.clone()
    }

    pub fn set_timer(&self, duration: Duration) -> Timer {
        self.0.upgrade().unwrap().borrow().set_timer(duration)
    }

    pub fn set_random_timer(&self, min_duration: Duration, max_duration: Duration) -> Timer {
        self.0
            .upgrade()
            .unwrap()
            .borrow()
            .set_random_timer(min_duration, max_duration)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn spawn<F>(&self, f: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send,
    {
        let f = SendSyncWrapper::new(f);
        self.0.upgrade().unwrap().borrow().spawn(f)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn name(&self) -> String {
        self.0.upgrade().unwrap().borrow().name.clone()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn resolve_addr(&self, addr: &Address) -> Option<SocketAddr> {
        self.0.upgrade().unwrap().borrow().net.get(addr).cloned()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents real node.
pub struct RealNode(Rc<RefCell<RealNodeState>>);

impl RealNode {
    /// Create new node with specified name, seed and routing config.
    /// Also mounting dirrectory for working with files must be specified.
    pub fn new(
        name: impl Into<String>,
        seed: u64,
        net: RouteConfig,
        mount_dir: impl Into<String>,
    ) -> Self {
        let state = RealNodeState::new(name, seed, net, mount_dir);
        RealNode(Rc::new(RefCell::new(state)))
    }

    /// Spawn async activity.
    pub fn spawn<F>(&self, f: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send,
    {
        self.0.borrow().spawn(f)
    }

    /// Block current thread until provided async
    /// activity not resolved.
    pub fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future,
    {
        let proc = self.0.borrow().proc.iter().nth(0).unwrap().1.clone();
        let proc_handle = ProcessHandle {
            proc: Rc::downgrade(&proc),
            node: RealNodeHandle(Rc::downgrade(&self.0)),
        };
        {
            let _guard = Guard::new(Context::new(proc_handle));
            let s = self.0.borrow();
            s.block_on(f)
        }
    }

    /// Add process on the node.
    /// Returns handles for send and receive local messages.
    pub fn add_proc(
        &mut self,
        name: impl Into<String>,
        proc: impl Process,
    ) -> Result<(LocalSender, LocalReceiver), Error> {
        let (sender, receiver) = unbounded_channel();
        let proc_name = name.into();
        let state = ProcessState::new(proc, proc_name.clone(), sender);
        let state = Rc::new(RefCell::new(state));
        let handle = ProcessHandle {
            proc: Rc::downgrade(&state),
            node: RealNodeHandle(Rc::downgrade(&self.0)),
        };

        let mut entry = self.0.borrow_mut();
        let entry = entry.proc.entry(proc_name);
        match entry {
            Entry::Occupied(_) => Err(Error::AlreadyExists),
            Entry::Vacant(e) => {
                e.insert(state);
                let sender = LocalSender { handle };
                let receiver = LocalReceiver { receiver };
                Ok((sender, receiver))
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::RealNodeState;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn spawn() {
        let state = RealNodeState::new("node", 123, Default::default(), String::default());
        let (sender, receiver) = tokio::sync::oneshot::channel();
        state.spawn(async move {
            sender.send(1).unwrap();
        });
        state.block_on(async move {
            let result = receiver.await.unwrap();
            assert_eq!(result, 1);
        })
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn set_timer() {
        let state = RealNodeState::new("node", 123, Default::default(), String::default());
        let start = Instant::now();
        let timer = state.set_timer(Duration::from_millis(20));
        state.block_on(async move {
            timer.await;
        });
        let duration = start.elapsed();
        assert!(duration >= Duration::from_millis(10));
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn set_random_timer() {
        let state = RealNodeState::new("node", 123, Default::default(), String::default());
        let start = Instant::now();
        let timer = state.set_random_timer(Duration::from_millis(10), Duration::from_millis(20));
        state.block_on(async move {
            timer.await;
        });
        let duration = start.elapsed();
        assert!(duration >= Duration::from_millis(5));
    }
}
