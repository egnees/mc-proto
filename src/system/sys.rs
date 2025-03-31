use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap, HashMap},
    hash::{DefaultHasher, Hasher},
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{
    event::{
        info::{Event, EventInfo},
        EventManager,
    },
    runtime::{Runtime, RuntimeHandle},
    search::{
        step::{Timer, UdpMessage},
        SearchConfig, SearchStep,
    },
    track::simple::SimpleTracker,
    util::oneshot,
};

use super::{
    context::{self, Guard},
    error::Error,
    log::{
        Log, LogEntry, ProcessReceivedLocalMessage, ProcessWokeUp, UdpMessageDropped,
        UdpMessageReceived,
    },
    net::{self, Network},
    node::Node,
    proc::Address,
    stat::Stat,
};

////////////////////////////////////////////////////////////////////////////////

pub type HashType = u64;

////////////////////////////////////////////////////////////////////////////////

pub struct State {
    pub nodes: BTreeMap<String, Node>,
    pub net: Network,
    pub events: EventManager,
    pub log: Log,
    pub time_from: Duration,
    pub time_to: Duration,
    pub timers: HashMap<usize, oneshot::Sender<bool>>,
    pub async_proc: HashMap<usize, Address>,
    pub stat: Stat,
    pub rt: RuntimeHandle,
}

////////////////////////////////////////////////////////////////////////////////

pub struct StateHandle(Weak<RefCell<State>>);

impl StateHandle {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self(Rc::downgrade(&state))
    }

    fn state(&self) -> Rc<RefCell<State>> {
        self.0.upgrade().unwrap()
    }

    pub fn read_locals(&self, proc: &Address) -> Option<Vec<String>> {
        self.state()
            .borrow()
            .nodes
            .get(&proc.node)
            .and_then(|n| n.read_locals(&proc.process))
            .cloned()
    }

    pub fn stat(&self) -> Stat {
        self.state().borrow().stat.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct System {
    rt: Runtime,
    state: Rc<RefCell<State>>,
}

impl System {
    pub fn new(net: &net::Config) -> Self {
        let events = EventManager::new(SimpleTracker::new());
        let rt = Runtime::default();
        let handle = rt.handle();
        let state = State {
            nodes: Default::default(),
            net: Network::new(net),
            events,
            log: Log::new(),
            time_from: Default::default(),
            time_to: Default::default(),
            timers: Default::default(),
            async_proc: Default::default(),
            stat: Stat::default(),
            rt: handle,
        };
        Self {
            rt,
            state: Rc::new(RefCell::new(state)),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn hash(&self) -> HashType {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(self, &mut hasher);
        hasher.finish()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn handle(&self) -> StateHandle {
        StateHandle::new(self.state.clone())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn log(&self) -> Log {
        self.state.borrow().log.clone()
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn context_guard(&self, cur_proc: Address) -> context::Guard {
        let ctx = context::Context {
            state: Rc::downgrade(&self.state),
            cur_proc,
            rt: self.rt.handle(),
        };
        Guard::new(ctx)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn send_local(&mut self, to: &Address, content: impl Into<String>) -> Result<(), Error> {
        let content = content.into();

        let log_entry = ProcessReceivedLocalMessage {
            process: to.clone(),
            content: content.clone(),
        };
        let log_entry = LogEntry::ProcessReceivedLocalMessage(log_entry);
        self.state.borrow_mut().log.add_entry(log_entry);

        {
            let proc = self
                .state
                .borrow()
                .nodes
                .get(&to.node)
                .and_then(|n| n.proc(&to.process))
                .ok_or(Error::NotFound)?;

            let _guard = self.context_guard(to.clone());
            proc.borrow_mut().on_local_message(content);
        }

        self.run_async();

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn apply_search_step(&mut self, step: &SearchStep) {
        match step {
            SearchStep::SelectUdp(id, udp_message) => {
                let event = self
                    .state
                    .borrow_mut()
                    .events
                    .remove_ready(*id)
                    .unwrap()
                    .clone();
                if udp_message.drop {
                    self.drop_udp_message(event);
                } else {
                    self.deliver_udp_message(event);
                }
            }
            SearchStep::SelectTimer(id, _) => {
                let event = self
                    .state
                    .borrow_mut()
                    .events
                    .remove_ready(*id)
                    .unwrap()
                    .clone();
                self.fire_timer(event);
            }
            SearchStep::Apply(apply_functor) => apply_functor.apply(self),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn search_steps(&self, cfg: &SearchConfig) -> Vec<SearchStep> {
        let state = self.state.borrow();
        let ready = state.events.ready_events_cnt();
        let mut res = Vec::new();
        for i in 0..ready {
            let e = state.events.get_ready(i).unwrap();
            match &e.info {
                EventInfo::UdpMessageInfo(udp) => {
                    let can_be_delivered = self.message_can_be_delivered(&udp.from, &udp.to);
                    if !can_be_delivered
                        || state.stat.udp_msg_dropped < cfg.max_msg_drops.unwrap_or(usize::MAX)
                    {
                        let udp_msg = UdpMessage {
                            udp_msg_id: udp.udp_msg_id,
                            drop: true,
                        };
                        let step = SearchStep::SelectUdp(i, udp_msg);
                        res.push(step);
                    }
                    if can_be_delivered {
                        let udp_msg = UdpMessage {
                            udp_msg_id: udp.udp_msg_id,
                            drop: false,
                        };
                        let step = SearchStep::SelectUdp(i, udp_msg);
                        res.push(step);
                    }
                }
                EventInfo::TimerInfo(timer) => {
                    let timer = Timer {
                        timer_id: timer.timer_id,
                    };
                    let step = SearchStep::SelectTimer(i, timer);
                    res.push(step);
                }
            }
        }
        res
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn fire_timer(&mut self, event: Event) {
        let time_from = event.time_from;
        let time_to = event.time_to;

        let timer = if let EventInfo::TimerInfo(timer) = event.info {
            timer
        } else {
            unreachable!()
        };

        // add entry to log
        let log_entry = if timer.with_sleep {
            let wokeup = ProcessWokeUp {
                proc: timer.proc.clone(),
            };
            LogEntry::ProcessWokeUp(wokeup)
        } else {
            unreachable!("basic timers without sleep is not supported")
        };

        // update state
        {
            let mut state = self.state.borrow_mut();
            state.log.add_entry(log_entry);
            state.time_from = time_from;
            state.time_to = time_to;
        }

        // wake up future associated with timer
        let sender_opt = self.state.borrow_mut().timers.remove(&timer.timer_id);
        if let Some(timer) = sender_opt {
            // receiver can be dropped which is ok
            let _ = timer.send(true);
        }

        // run async
        self.run_async();
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn message_can_be_delivered(&self, from: &Address, to: &Address) -> bool {
        let state = self.state.borrow();
        let from_present = state
            .nodes
            .get(&from.node)
            .and_then(|n| n.proc(&from.process))
            .is_some();
        if !from_present {
            return false;
        }
        let to_present = state
            .nodes
            .get(&to.node)
            .and_then(|n| n.proc(&to.process))
            .is_some();
        to_present
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn deliver_udp_message(&mut self, event: Event) {
        // get upd message info
        let time_from = event.time_from;
        let time_to = event.time_to;

        let msg_info = if let EventInfo::UdpMessageInfo(msg_info) = event.info.clone() {
            msg_info
        } else {
            unreachable!()
        };

        if !self.message_can_be_delivered(&msg_info.from, &msg_info.to) {
            self.drop_udp_message(event);
        }

        // get receiver process
        let proc = self
            .state
            .borrow()
            .nodes
            .get(&msg_info.to.node)
            .unwrap()
            .proc(&msg_info.to.process)
            .unwrap();

        let log_entry = UdpMessageReceived {
            from: msg_info.from.clone(),
            to: msg_info.to.clone(),
            content: msg_info.content.clone(),
        };
        let log_entry = LogEntry::UdpMessageReceived(log_entry);

        {
            let mut state = self.state.borrow_mut();

            // add entry to log
            state.log.add_entry(log_entry);

            // update stat
            state.stat.udp_msg_delivered += 1;

            // update time
            state.time_from = time_from;
            state.time_to = time_to;
        }

        {
            // set context guard
            let _guard = self.context_guard(msg_info.to.clone());

            // deliver message
            proc.borrow_mut()
                .on_message(msg_info.from, msg_info.content);
        }

        // run async
        self.run_async();
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn drop_udp_message(&mut self, event: Event) {
        let time_from = event.time_from;
        let time_to = event.time_to;

        let msg_info = if let EventInfo::UdpMessageInfo(msg_info) = event.info {
            msg_info
        } else {
            unreachable!()
        };

        // add entry to log
        let log_entry = UdpMessageDropped {
            from: msg_info.from.clone(),
            to: msg_info.to,
            content: msg_info.content.clone(),
        };
        let log_entry = LogEntry::UdpMessageDropped(log_entry);

        {
            let mut state = self.state.borrow_mut();

            // update log
            state.log.add_entry(log_entry);

            // update stat
            state.stat.udp_msg_dropped += 1;

            // update time
            state.time_from = time_from;
            state.time_to = time_to;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn add_node(&mut self, name: impl Into<String>, node: Node) -> Result<(), Error> {
        let name: String = name.into();
        if let Entry::Vacant(e) = self.state.borrow_mut().nodes.entry(name) {
            e.insert(node);
            Ok(())
        } else {
            Err(Error::AlreadyExists)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn drain_locals(&mut self, addr: &Address) -> Option<Vec<String>> {
        self.state
            .borrow_mut()
            .nodes
            .get_mut(&addr.node)
            .and_then(|n| n.get_locals(&addr.process))
            .map(std::mem::take)
    }

    pub fn read_locals(&self, addr: &Address) -> Option<Vec<String>> {
        self.state
            .borrow()
            .nodes
            .get(&addr.node)
            .and_then(|n| n.read_locals(&addr.process))
            .cloned()
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn run_async(&self) {
        while let Some(next_id) = self.rt.next_task_id() {
            let proc = self
                .state
                .borrow()
                .async_proc
                .get(&next_id)
                .unwrap()
                .clone();
            let _guard = self.context_guard(proc);
            self.rt.process_next_task();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl std::hash::Hash for System {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.borrow().events.hash(state);
        self.state
            .borrow()
            .nodes
            .iter()
            .for_each(|(_name, node)| node.hash(state));
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        search::{step::UdpMessage, SearchStep},
        system::{
            net::{send_message, Config},
            node::Node,
            proc::{send_local, Address, Process},
        },
    };

    use super::{HashType, System};

    ////////////////////////////////////////////////////////////////////////////////

    struct Proc {
        second: Address,
    }

    impl Process for Proc {
        fn on_message(&mut self, _from: Address, content: String) {
            send_local(content);
        }

        fn on_local_message(&mut self, content: String) {
            send_message(&self.second, content);
        }

        fn hash(&self) -> HashType {
            unreachable!()
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut sys = System::new(
            &Config::new(Duration::from_secs_f64(0.1), Duration::from_secs_f64(0.2)).unwrap(),
        );

        let mut n1 = Node::new();
        n1.add_proc(
            "p1",
            Proc {
                second: Address::new("n2", "p2"),
            },
        )
        .unwrap();
        sys.add_node("n1", n1).unwrap();

        let mut n2 = Node::new();
        n2.add_proc(
            "p2",
            Proc {
                second: Address::new("n1", "p1"),
            },
        )
        .unwrap();
        sys.add_node("n2", n2).unwrap();

        sys.send_local(&Address::new("n1", "p1"), "m").unwrap();

        sys.apply_search_step(&SearchStep::SelectUdp(
            0,
            UdpMessage {
                udp_msg_id: 0,
                drop: false,
            },
        ));

        let locals = sys.drain_locals(&Address::new("n2", "p2")).unwrap();
        assert_eq!(locals, vec!["m".to_string()]);
    }
}
