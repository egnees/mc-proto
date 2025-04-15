use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    future::Future,
    hash::Hash,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{
    event::driver::EventDriver,
    runtime::{JoinHandle, RuntimeHandle},
    simulation::{
        context::{Context, Guard},
        log::{
            FutureFellAsleep, FutureWokeUp, Log, LogEntry, ProcessReceivedLocalMessage,
            ProcessSentLocalMessage, UdpMessageDropped, UdpMessageReceived, UdpMessageSent,
        },
        proc::ProcessHandle,
    },
    util::oneshot::Sender,
    Address, SystemHandle,
};

use super::{
    info::{EventInfo, Timer, UdpMessage},
    outcome::{EventOutcome, EventOutcomeKind},
    stat::EventStat,
    time::TimeSegment,
    Event,
};

////////////////////////////////////////////////////////////////////////////////

pub struct EventManagerState {
    system: Option<SystemHandle>,
    rt: RuntimeHandle,
    events: Vec<Event>,
    time: TimeSegment,
    event_log: Log,
    driver: Weak<RefCell<dyn EventDriver>>,
    timers: HashMap<usize, Sender<bool>>,
    next_udp_msg_id: usize,
    next_timer_id: usize,
    stat: EventStat,
    unhandled_events: BTreeSet<usize>,
}

impl EventManagerState {
    fn inc_udp_msg_id(&mut self) -> usize {
        let res = self.next_udp_msg_id;
        self.next_udp_msg_id += 1;
        res
    }

    fn inc_timer_id(&mut self) -> usize {
        let res = self.next_timer_id;
        self.next_timer_id += 1;
        res
    }

    fn register_in_driver(&mut self, event: &Event) {
        self.driver
            .upgrade()
            .expect("can not upgrade driver")
            .borrow_mut()
            .register_event(event);
        self.unhandled_events.insert(event.id);
    }

    fn system(&self) -> SystemHandle {
        self.system.as_ref().unwrap().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct EventManager(Rc<RefCell<EventManagerState>>);

impl EventManager {
    pub fn new(rt: RuntimeHandle, driver: &Rc<RefCell<dyn EventDriver>>) -> Self {
        let state = EventManagerState {
            system: None,
            rt,
            events: Default::default(),
            event_log: Default::default(),
            time: Default::default(),
            driver: Rc::downgrade(driver),
            timers: Default::default(),
            next_udp_msg_id: 0,
            next_timer_id: 0,
            stat: Default::default(),
            unhandled_events: Default::default(),
        };
        Self(Rc::new(RefCell::new(state)))
    }

    pub fn handle(&self) -> EventManagerHandle {
        EventManagerHandle(Rc::downgrade(&self.0))
    }

    pub fn set_system_handle(&self, handle: SystemHandle) {
        self.0.borrow_mut().system = Some(handle);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct EventManagerHandle(Weak<RefCell<EventManagerState>>);

impl EventManagerHandle {
    fn state(&self) -> Rc<RefCell<EventManagerState>> {
        self.0.upgrade().expect("can not upgrade manager handle")
    }

    fn guard(&self, proc: ProcessHandle) -> Guard {
        let ctx = Context {
            event_manager: self.clone(),
            proc,
        };
        Guard::new(ctx)
    }

    pub fn time(&self) -> TimeSegment {
        self.state().borrow().time
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn log(&self) -> Log {
        self.state().borrow().event_log.clone()
    }

    pub fn stat(&self) -> EventStat {
        self.state().borrow().stat.clone()
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Register events
    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_udp_message(&self, from: ProcessHandle, to: &Address, content: String) {
        let state = self.state();
        let mut state = state.borrow_mut();

        // add log entry
        {
            let sent_entry = UdpMessageSent {
                from: from.address(),
                to: to.clone(),
                content: content.clone(),
                time: state.time,
            };
            let log_entry = LogEntry::UdpMessageSent(sent_entry);
            state.event_log.add_entry(log_entry);
        }

        // get receiver process
        let system_handle = state.system();
        let Some(to) = system_handle.proc_by_addr(to) else {
            // no such process
            let dropped_entry = UdpMessageDropped {
                from: from.address(),
                to: to.clone(),
                content,
                time: state.time,
            };
            let log_entry = LogEntry::UdpMessageDropped(dropped_entry);
            state.event_log.add_entry(log_entry);
            return;
        };

        // create udp event
        let info = UdpMessage {
            udp_msg_id: state.inc_udp_msg_id(),
            from,
            to,
            content,
        };
        let info = EventInfo::UdpMessage(info);
        let net = system_handle.network();
        let event = Event {
            id: state.events.len(),
            time: state
                .time
                .shift_range(net.min_packet_delay, net.max_packet_delay),
            info,
        };

        // register event
        state.register_in_driver(&event);
        state.events.push(event);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_sleep(&self, proc: ProcessHandle, on: Duration, sender: Sender<bool>) {
        let state = self.state();
        let mut state = state.borrow_mut();

        let timer_id = state.inc_timer_id();
        let insert_result = state.timers.insert(timer_id, sender);
        assert!(insert_result.is_none());

        // add log entry
        {
            let sleep_entry = FutureFellAsleep {
                tag: timer_id,
                proc: proc.address(),
                duration: on,
                time: state.time,
            };
            let log_entry = LogEntry::FutureFellAsleep(sleep_entry);
            state.event_log.add_entry(log_entry);
        }

        // make event
        let info = Timer {
            timer_id,
            proc,
            duration: on,
            with_sleep: true,
        };
        let info = EventInfo::Timer(info);
        let event = Event {
            id: state.events.len(),
            time: state.time.shift(on),
            info,
        };
        state.register_in_driver(&event);
        state.events.push(event);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_local_msg_from_process(&self, proc: ProcessHandle, content: String) {
        // log
        {
            let state = self.state();
            let mut state = state.borrow_mut();

            let log_entry = ProcessSentLocalMessage {
                process: proc.address(),
                content: content.clone(),
                time: state.time,
            };
            let log_entry = LogEntry::ProcessSentLocalMessage(log_entry);
            state.event_log.add_entry(log_entry);
        }

        // store local
        proc.store_local(content);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_async_task<F: Future + 'static>(
        &self,
        task: F,
        owner: ProcessHandle,
    ) -> JoinHandle<F::Output> {
        self.state().borrow().rt.spawn(task, owner)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Handle event outcome
    ////////////////////////////////////////////////////////////////////////////////

    pub fn handle_event_outcome(&self, outcome: &EventOutcome) {
        // update state and get event
        let event = {
            let state = self.state();
            let mut state = state.borrow_mut();

            // remove from unhandled
            let remove_result = state.unhandled_events.remove(&outcome.event_id);
            assert!(remove_result);

            // get event
            let event_id = outcome.event_id;
            let event = state.events[event_id].clone();

            // update time
            state.time = event.time;

            // return event
            event
        };
        match outcome.kind {
            EventOutcomeKind::UdpMessageDropped() => self.handle_udp_message_dropped(&event),
            EventOutcomeKind::UdpMessageDelivered() => self.handle_udp_message_delivered(&event),
            EventOutcomeKind::TimerFired() => self.handle_timer_fired(&event),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Handle outcomes
    ////////////////////////////////////////////////////////////////////////////////

    fn handle_udp_message_dropped(&self, event: &Event) {
        let msg = variant::variant!(&event.info, EventInfo::UdpMessage(msg));

        // add log entry
        let state = self.state();
        let mut state = state.borrow_mut();

        let dropped_entry = UdpMessageDropped {
            from: msg.from.address(),
            to: msg.to.address(),
            content: msg.content.clone(),
            time: event.time,
        };
        let log_entry = LogEntry::UdpMessageDropped(dropped_entry);
        state.event_log.add_entry(log_entry);
        state.stat.udp_msg_dropped += 1;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn handle_udp_message_delivered(&self, event: &Event) {
        let msg = variant::variant!(&event.info, EventInfo::UdpMessage(msg));

        // add log entry
        let recv = {
            let state = self.state();
            let mut state = state.borrow_mut();

            let received_entry = UdpMessageReceived {
                from: msg.from.address(),
                to: msg.to.address(),
                content: msg.content.clone(),
                time: event.time,
            };
            let log_entry = LogEntry::UdpMessageReceived(received_entry);
            state.event_log.add_entry(log_entry);

            state
                .system()
                .proc_by_addr(&msg.to.address())
                .expect("trying to deliver msg to dead process")
                .proc()
        };

        // callback
        let _guard = self.guard(msg.to.clone());
        recv.borrow_mut()
            .on_message(msg.from.address(), msg.content.clone());
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn handle_timer_fired(&self, event: &Event) {
        let timer = variant::variant!(&event.info, EventInfo::Timer(timer));
        assert!(timer.with_sleep);

        // add log entry
        let sender = {
            let state = self.state();
            let mut state = state.borrow_mut();

            let wakeup_entry = FutureWokeUp {
                tag: timer.timer_id,
                proc: timer.proc.address(),
                time: event.time,
            };
            let log_entry = LogEntry::FutureWokeUp(wakeup_entry);
            state.event_log.add_entry(log_entry);

            // get sender
            state
                .timers
                .remove(&timer.timer_id)
                .expect("trying to handle not registered timer")
        };

        // wakeup sleeping future
        let _ = sender.send(true);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn handle_local_msg_from_user(&self, proc: ProcessHandle, content: String) {
        // log
        {
            let state = self.state();
            let mut state = state.borrow_mut();

            let log_entry = ProcessReceivedLocalMessage {
                process: proc.address(),
                content: content.clone(),
                time: state.time,
            };
            let log_entry = LogEntry::ProcessReceivedLocalMessage(log_entry);
            state.event_log.add_entry(log_entry);
        }

        // store local
        let _guard = self.guard(proc.clone());
        proc.proc().borrow_mut().on_local_message(content);
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Hash for EventManager {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let manager_state = self.0.borrow();
        manager_state
            .unhandled_events
            .iter()
            .map(|e| &manager_state.events[*e])
            .for_each(|e| e.hash(state));
    }
}
