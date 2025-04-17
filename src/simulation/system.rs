use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{
    event::{
        driver::EventDriver, manager::EventManager, outcome::EventOutcome, stat::EventStat,
        time::TimeSegment,
    },
    runtime::Runtime,
    NetConfig,
};

use super::{
    context::{Context, Guard},
    error::Error,
    log::Log,
    net::Network,
    node::Node,
    proc::{Address, ProcessHandle},
};

////////////////////////////////////////////////////////////////////////////////

pub type HashType = u64;

////////////////////////////////////////////////////////////////////////////////

struct SystemState {
    nodes: BTreeMap<String, Node>,
    net: Network,
    rt: Runtime,
    event_manager: EventManager,
}

impl Hash for SystemState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.values().for_each(|n| n.hash(state));
        self.event_manager.hash(state);
    }
}

impl SystemState {
    fn new_ref(net: &NetConfig, driver: &Rc<RefCell<dyn EventDriver>>) -> Rc<RefCell<Self>> {
        let net = Network::new(net);
        let rt = Runtime::default();
        let event_manager = EventManager::new(rt.handle(), driver);
        let sys_state = SystemState {
            nodes: Default::default(),
            net,
            rt,
            event_manager,
        };
        let state_ref = Rc::new(RefCell::new(sys_state));
        let handle = SystemHandle(Rc::downgrade(&state_ref));
        state_ref.borrow().event_manager.set_system_handle(handle);
        state_ref
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

impl Hash for SystemHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state().borrow().hash(state);
    }
}

impl SystemHandle {
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

    pub(crate) fn network(&self) -> Network {
        self.state().borrow().net.clone()
    }

    pub(crate) fn hash(&self) -> HashType {
        let mut hasher = std::hash::DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        hasher.finish()
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn guard(&self, proc: ProcessHandle) -> Guard {
        let ctx = Context {
            event_manager: self.state().borrow().event_manager.handle(),
            proc,
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

    pub fn set_network_delays(&self, min: Duration, max: Duration) -> Result<(), Error> {
        if min > max {
            return Err(Error::IncorrectRange);
        }
        let state = self.state();
        let net = &mut state.borrow_mut().net;
        net.min_packet_delay = min;
        net.max_packet_delay = max;
        Ok(())
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
        if let Entry::Vacant(e) = self.state().borrow_mut().nodes.entry(node.name.clone()) {
            e.insert(node);
            Ok(())
        } else {
            Err(Error::AlreadyExists)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn log(&self) -> Log {
        self.state().borrow().event_manager.handle().log()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn time(&self) -> TimeSegment {
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
}

// pub(crate) fn apply_search_step(&mut self, step: &SearchStep) {
//     match step {
//         SearchStep::SelectUdp(id, udp_message) => {
//             let event = self
//                 .system
//                 .borrow_mut()
//                 .events
//                 .remove_ready(*id)
//                 .unwrap()
//                 .clone();
//             if udp_message.drop {
//                 self.drop_udp_message(event);
//             } else {
//                 self.deliver_udp_message(event);
//             }
//         }
//         SearchStep::SelectTimer(id, _) => {
//             let event = self
//                 .system
//                 .borrow_mut()
//                 .events
//                 .remove_ready(*id)
//                 .unwrap()
//                 .clone();
//             self.fire_timer(event);
//         }
//         SearchStep::Apply(apply_functor) => apply_functor.apply(self),
//     }
// }

// ////////////////////////////////////////////////////////////////////////////////

// pub(crate) fn search_steps(&self, cfg: &SearchConfig) -> Vec<SearchStep> {
//     let state = self.system.borrow();
//     let ready = state.events.ready_events_cnt();
//     let mut res = Vec::new();
//     for i in 0..ready {
//         let e = state.events.get_ready(i).unwrap();
//         match &e.info {
//             EventInfo::UdpMessageInfo(udp) => {
//                 let can_be_delivered = self.message_can_be_delivered(&udp.from, &udp.to);
//                 if !can_be_delivered
//                     || state.stat.udp_msg_dropped < cfg.max_msg_drops.unwrap_or(usize::MAX)
//                 {
//                     let udp_msg = UdpMessage {
//                         udp_msg_id: udp.udp_msg_id,
//                         drop: true,
//                     };
//                     let step = SearchStep::SelectUdp(i, udp_msg);
//                     res.push(step);
//                 }
//                 if can_be_delivered {
//                     let udp_msg = UdpMessage {
//                         udp_msg_id: udp.udp_msg_id,
//                         drop: false,
//                     };
//                     let step = SearchStep::SelectUdp(i, udp_msg);
//                     res.push(step);
//                 }
//             }
//             EventInfo::TimerInfo(timer) => {
//                 let timer = Timer {
//                     timer_id: timer.timer_id,
//                 };
//                 let step = SearchStep::SelectTimer(i, timer);
//                 res.push(step);
//             }
//         }
//     }
//     res
// }
