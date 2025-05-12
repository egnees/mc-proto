use std::{
    any::Any,
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    rc::{Rc, Weak},
};

use crate::{
    event::{
        driver::EventDriver, manager::EventManager, outcome::EventOutcome, stat::EventStat,
        time::Time,
    },
    fs::manager::FsManagerHandle,
    runtime::Runtime,
    util, NetConfig,
};

use super::{
    context::{Context, Guard},
    error::Error,
    hash::HashContext,
    log::Log,
    net::{Network, NetworkHandle},
    node::{Node, NodeRoleRegister},
    proc::{Address, ProcessHandle},
};

////////////////////////////////////////////////////////////////////////////////

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
        util::hash::hash_list([nodes_hash, events_hash].into_iter())
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

    pub fn network(&self) -> NetworkHandle {
        self.state().borrow().net.handle()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn send_local(&self, to: &Address, content: impl Into<String>) -> Result<(), Error> {
        let content = content.into();

        let event_manager = self.state().borrow().event_manager.handle();
        let proc = self.proc_by_addr(to).ok_or(Error::NotFound)?;
        event_manager.handle_local_msg_from_user(proc, content);

        self.run_async_tasks();
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn read_locals(
        &self,
        node: impl Into<String>,
        proc: impl Into<String>,
    ) -> Option<Vec<String>> {
        let addr = Address::new(node, proc);
        Some(self.proc_by_addr(&addr)?.read_locals())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn drain_locals(&self, proc: &Address) -> Option<Vec<String>> {
        Some(self.proc_by_addr(proc)?.drain_locals())
    }

    ////////////////////////////////////////////////////////////////////////////////

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

    pub fn setup_fs(
        &self,
        node: impl Into<String>,
        delays: Time,
        capacity: usize,
    ) -> Result<(), Error> {
        let node = node.into();
        let reg = self.state().borrow().event_manager.handle().fs_registry();
        self.state()
            .borrow_mut()
            .nodes
            .get_mut(&node)
            .ok_or(Error::NotFound)?
            .setup_fs(reg, delays, capacity)
    }

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

    pub fn shutdown_fs(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();
        self.state()
            .borrow_mut()
            .nodes
            .get_mut(&node)
            .ok_or(Error::NotFound)?
            .shutdown_fs()
    }

    pub fn shutdown_node(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();

        self.state()
            .borrow()
            .event_manager
            .handle()
            .on_node_crash(node.as_str());

        let rt = self.state().borrow().rt.handle();
        rt.cancel_tasks(|p| p.address().node == node);

        self.state()
            .borrow_mut()
            .nodes
            .get_mut(node.as_str())
            .ok_or(Error::NotFound)?
            .shutdown();

        self.run_async_tasks();

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn log(&self) -> Log {
        self.state().borrow().event_manager.handle().log()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn time(&self) -> Time {
        self.state().borrow().event_manager.handle().time()
    }

    ////////////////////////////////////////////////////////////////////////////////

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

    pub fn pending_events(&self) -> usize {
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

    pub fn crash_node(&self, node: impl Into<String>) -> Result<(), Error> {
        let node = node.into();

        self.state()
            .borrow()
            .event_manager
            .handle()
            .on_node_crash(node.as_str());

        let rt = self.state().borrow().rt.handle();
        rt.cancel_tasks(|p| p.try_address().map(|a| a.node == node).unwrap_or(false));

        let node = self
            .state()
            .borrow_mut()
            .nodes
            .remove(node.as_str())
            .ok_or(Error::NotFound)?;

        drop(node);

        self.run_async_tasks();

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn proc(&self, addr: impl Into<Address>) -> Option<ProcessHandle> {
        let addr: Address = addr.into();
        self.proc_by_addr(&addr)
    }

    pub fn proc_state<T: Any>(&self, addr: impl Into<Address>) -> Option<Rc<RefCell<T>>> {
        self.proc(addr).and_then(|p| p.proc_state::<T>())
    }
}
