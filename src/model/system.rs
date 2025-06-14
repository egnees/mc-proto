use std::{
    any::Any,
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    rc::{Rc, Weak},
    time::Duration,
};

use crate::model::{
    event::{driver::EventDriver, manager::EventManager, outcome::EventOutcome, stat::EventStat},
    fs::manager::FsManagerHandle,
    net::Config as NetConfig,
    runtime::Runtime,
};

pub use crate::{Address, Process};

use super::{
    context::{Context, Guard},
    error::Error,
    hash::HashContext,
    log::Log,
    net::{Network, NetworkHandle},
    node::{Node, NodeRoleRegister},
    proc::ProcessHandle,
};

////////////////////////////////////////////////////////////////////////////////

/// Type of hash of the system model state.
pub type HashType = u64;

////////////////////////////////////////////////////////////////////////////////

struct SystemState {
    nodes: BTreeMap<String, Node>,
    roles: NodeRoleRegister,
    net: Network,
    rt: Runtime,
    event_manager: EventManager,
}

impl SystemState {
    fn new_ref(net: &NetConfig, driver: &Rc<RefCell<dyn EventDriver>>) -> Rc<RefCell<Self>> {
        let net = Network::new(net);
        let rt = Runtime::default();
        let event_manager = EventManager::new(rt.handle(), driver);
        let sys_state = SystemState {
            nodes: Default::default(),
            roles: Default::default(),
            net,
            rt,
            event_manager,
        };
        let state_ref = Rc::new(RefCell::new(sys_state));
        let handle = SystemHandle(Rc::downgrade(&state_ref));
        state_ref.borrow().event_manager.set_system_handle(handle);
        state_ref
    }

    fn hash(&self) -> HashType {
        let ctx = HashContext::new(&self.roles);
        let nodes_hash = ctx.hash_nodes(self.nodes.values());
        let events_hash = self.event_manager.hash(ctx);
        nodes_hash ^ events_hash
        // util::hash::hash_list([nodes_hash, events_hash].into_iter())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct System(Rc<RefCell<SystemState>>);

impl System {
    pub(crate) fn new_default_net(driver: &Rc<RefCell<dyn EventDriver>>) -> Self {
        Self::new(&NetConfig::default(), driver)
    }

    pub(crate) fn new(net: &NetConfig, driver: &Rc<RefCell<dyn EventDriver>>) -> Self {
        let state = SystemState::new_ref(net, driver);
        Self(state)
    }

    pub(crate) fn handle(&self) -> SystemHandle {
        SystemHandle(Rc::downgrade(&self.0))
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents handle on the system model.
#[derive(Clone)]
pub struct SystemHandle(Weak<RefCell<SystemState>>);

impl SystemHandle {
    pub(crate) fn system_dropped(&self) -> bool {
        self.0.strong_count() == 0
    }

    fn state(&self) -> Rc<RefCell<SystemState>> {
        self.0.upgrade().unwrap()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn proc_by_addr(&self, addr: &Address) -> Option<ProcessHandle> {
        self.state()
            .borrow()
            .nodes
            .get(&addr.node)?
            .proc(&addr.process)
    }

    /// Get hash of the system model state.
    pub fn hash(&self) -> HashType {
        self.state().borrow().hash()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn fs(&self, node: &str) -> Option<FsManagerHandle> {
        self.state()
            .borrow()
            .nodes
            .get(node)
            .unwrap()
            .fs
            .as_ref()
            .map(|fs| fs.handle())
    }

    fn guard(&self, proc: ProcessHandle) -> Guard {
        let fs = if proc.alive() {
            self.fs(&proc.address().node)
        } else {
            None
        };

        let ctx = Context {
            event_manager: self.state().borrow().event_manager.handle(),
            proc,
            fs,
        };

        Guard::new(ctx)
    }

    pub(crate) fn run_async_tasks(&self) {
        while let Some(task_owner) = self.state().borrow().rt.next_task_owner() {
            let _guard = self.guard(task_owner);
            self.state().borrow().rt.process_next_task();
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Allows to get handle on the network.
    pub fn network(&self) -> NetworkHandle {
        self.state().borrow().net.handle()
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Send local message to the process.
    pub fn send_local(&self, to: &Address, content: impl Into<String>) -> Result<(), Error> {
        let content = content.into();

        let event_manager = self.state().borrow().event_manager.handle();
        let proc = self.proc_by_addr(to).ok_or(Error::NotFound)?;
        event_manager.handle_local_msg_from_user(proc, content);

        self.run_async_tasks();
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Read local messages sent by process.
    pub fn read_locals(
        &self,
        node: impl Into<String>,
        proc: impl Into<String>,
    ) -> Option<Vec<String>> {
        let addr = Address::new(node, proc);
        Some(self.proc_by_addr(&addr)?.read_locals())
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Drain local messages sent by process.
    pub fn drain_locals(&self, proc: &Address) -> Option<Vec<String>> {
        Some(self.proc_by_addr(proc)?.drain_locals())
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Add node in the system.
    pub fn add_node(&self, node: Node) -> Result<(), Error> {
        let state = self.state();
        let mut state = state.borrow_mut();
        if let Entry::Vacant(e) = state.nodes.entry(node.name.clone()) {
            e.insert(node);
            Ok(())
        } else {
            Err(Error::AlreadyExists)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Add node with specified role.
    pub fn add_node_with_role(&self, node: Node, role: impl Into<String>) -> Result<(), Error> {
        let state = self.state();
        let mut state = state.borrow_mut();
        let name = node.name.clone();
        let added = if let Entry::Vacant(e) = state.nodes.entry(node.name.clone()) {
            e.insert(node);
            true
        } else {
            false
        };
        if added {
            state.roles.add(name, role);
            Ok(())
        } else {
            Err(Error::AlreadyExists)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Setup filesystem of the node.
    pub fn setup_fs(
        &self,
        node: impl Into<String>,
        min_delay: Duration,
        max_delay: Duration,
        capacity: usize,
    ) -> Result<(), Error> {
        let node = node.into();
        let reg = self.state().borrow().event_manager.handle().fs_registry();
        self.state()
            .borrow_mut()
            .nodes
            .get_mut(&node)
            .ok_or(Error::NotFound)?
            .setup_fs(reg, min_delay, max_delay, capacity)
    }

    /// Crash node file system.
    pub fn crash_fs(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();
        self.state()
            .borrow_mut()
            .nodes
            .get_mut(&node)
            .ok_or(Error::NotFound)?
            .crash_fs();
        Ok(())
    }

    /// Shutdown node filesy stem.
    pub fn shutdown_fs(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();
        self.state()
            .borrow_mut()
            .nodes
            .get_mut(&node)
            .ok_or(Error::NotFound)?
            .shutdown_fs()
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Get event log.
    pub fn log(&self) -> Log {
        self.state().borrow().event_manager.handle().log()
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Get current time in the system.
    pub fn time(&self) -> Duration {
        self.state().borrow().event_manager.handle().time()
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Get statistics of the events.
    pub fn stat(&self) -> EventStat {
        self.state().borrow().event_manager.handle().stat()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn handle_event_outcome(&self, outcome: EventOutcome) {
        let event_manager = self.state().borrow().event_manager.handle();
        event_manager.handle_event_outcome(&outcome);
        self.run_async_tasks();
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[allow(unused)]
    pub(crate) fn pending_events(&self) -> usize {
        self.state()
            .borrow()
            .event_manager
            .handle()
            .pending_events()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn nodes_count(&self) -> usize {
        self.state().borrow().nodes.len()
    }

    pub(crate) fn crash_node_index(&self, i: usize) {
        let key = self.state().borrow().nodes.keys().nth(i).cloned().unwrap();
        self.crash_node(key).unwrap();
    }

    pub(crate) fn node_available_index(&self, i: usize) -> bool {
        let key = self.state().borrow().nodes.keys().nth(i).cloned().unwrap();
        let result = self.state().borrow().nodes.get(&key).unwrap().shutdown;
        !result
    }

    /// Allows to crash node.
    /// File system clears.
    pub fn crash_node(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();

        self.state().borrow_mut().roles.remove(&node);

        let n = self
            .state()
            .borrow_mut()
            .nodes
            .remove(node.as_str())
            .ok_or(Error::NotFound)?;

        self.state()
            .borrow()
            .event_manager
            .handle()
            .on_node_crash(node.as_str());

        let rt = self.state().borrow().rt.handle();
        rt.cancel_tasks(|p| p.try_address().map(|a| a.node == node).unwrap_or(false));

        self.run_async_tasks();

        drop(n);

        // after that no async tasks connected
        // with node processes should be

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn shutdown_node_index(&self, id: usize) {
        let node = self.state().borrow().nodes.keys().nth(id).cloned().unwrap();
        self.shutdown_node(node).unwrap()
    }

    /// Allows to shutdown node.
    /// File system is preserved.
    pub fn shutdown_node(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();

        let role = self.state().borrow_mut().roles.remove(&node);

        let mut n = self
            .state()
            .borrow_mut()
            .nodes
            .remove(node.as_str())
            .ok_or(Error::NotFound)?;

        let _ = n.shutdown_fs();
        let fs = n.fs.take();

        self.state()
            .borrow()
            .event_manager
            .handle()
            .on_node_shutdown(node.as_str());

        let rt = self.state().borrow().rt.handle();
        rt.cancel_tasks(|p| p.try_address().map(|a| a.node == node).unwrap_or(false));

        // after that no async tasks connected
        // with node processes should be

        let mut name = String::new();
        std::mem::swap(&mut name, &mut n.name);

        drop(n);

        let mut n = Node::new(name);
        n.shutdown = true;
        n.fs = fs;

        if let Some(role) = role {
            self.add_node_with_role(n, role).unwrap();
        } else {
            self.add_node(n).unwrap();
        }

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Allows to restart shutdown node.
    pub fn restart_node(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();
        let state = self.state();
        let mut state = state.borrow_mut();
        let node = state.nodes.get_mut(&node).ok_or(Error::NotFound)?;
        node.shutdown = false;
        if let Some(fs) = node.fs.as_ref() {
            fs.handle().raise();
        }
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Allows to add process on node.
    pub fn add_proc_on_node(
        &self,
        node_name: impl Into<String>,
        proc_name: impl Into<String>,
        proc: impl Process,
    ) -> Result<ProcessHandle, Error> {
        let node = node_name.into();
        let state = self.state();
        let mut state = state.borrow_mut();
        let node = state.nodes.get_mut(&node).ok_or(Error::NotFound)?;
        if node.shutdown {
            Err(Error::NodeUnavailable)
        } else {
            node.add_proc(proc_name, proc)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Get process handle.
    pub fn proc(&self, addr: impl Into<Address>) -> Option<ProcessHandle> {
        let addr: Address = addr.into();
        self.proc_by_addr(&addr)
    }

    /// Get state of the process.
    pub fn proc_state<T: Any>(&self, addr: impl Into<Address>) -> Option<Rc<RefCell<T>>> {
        self.proc(addr).and_then(|p| p.proc_state::<T>())
    }
}
