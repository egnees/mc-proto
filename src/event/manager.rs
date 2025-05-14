use std::{
    cell::RefCell,
    collections::BTreeSet,
    future::Future,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{
    event::{
        driver::EventDriver,
        info::{RpcMessage, RpcMessageKind, TcpMessage},
    },
    fs::{event::FsEvent, registry::FsEventRegistry},
    rpc::{RpcListener, RpcManager, RpcRegistry, RpcRequest, RpcResponse, RpcResult},
    runtime::{JoinHandle, RuntimeHandle},
    sim::{
        context::{Context, Guard},
        hash::HashContext,
        log::{
            FutureFellAsleep, FutureWokeUp, Log, LogEntry, NodeCrashed, ProcessInfo,
            ProcessReceivedLocalMessage, ProcessSentLocalMessage, RpcMessageDropped,
            RpcMessageReceived, RpcMessageSent, TcpMessageDropped, TcpMessageReceived,
            TcpMessageSent, TimerCanceled, TimerFired, TimerSet, UdpMessageDropped,
            UdpMessageReceived, UdpMessageSent,
        },
        proc::{ProcessHandle, ProcessState},
    },
    tcp::{
        error::TcpError,
        manager::TcpConnectionManager,
        packet::TcpPacket,
        registry::TcpRegistry,
        stream::{TcpSender, TcpStream},
    },
    time,
    timer::{manager::TimerManager, registry::TimerRegistry},
    util::{
        oneshot,
        trigger::{self, make_trigger, Trigger},
    },
    Address, HashType, Process, SystemHandle,
};

use super::{
    info::{EventInfo, RpcEvent, RpcEventKind, TcpEvent, TcpEventKind, Timer, UdpMessage},
    outcome::{EventOutcome, EventOutcomeKind},
    stat::EventStat,
    time::Time,
    Event,
};

////////////////////////////////////////////////////////////////////////////////

pub struct EventManagerState {
    system: Option<SystemHandle>,
    rt: RuntimeHandle,
    events: Vec<Event>,
    time: Time,
    event_log: Rc<RefCell<Log>>,
    driver: Weak<RefCell<dyn EventDriver>>,
    next_udp_msg_id: usize,
    next_tcp_msg_id: usize,
    next_tcp_stream_id: usize,
    stat: EventStat,
    unhandled_events: BTreeSet<usize>,
    tcp: TcpConnectionManager,
    rpc: Rc<RefCell<RpcManager>>,
    timers: TimerManager,
}

impl EventManagerState {
    fn hash(&self, ctx: HashContext) -> HashType {
        ctx.hash_events(self.unhandled_events.iter().map(|e| &self.events[*e]))
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn inc_udp_msg_id(&mut self) -> usize {
        let res = self.next_udp_msg_id;
        self.next_udp_msg_id += 1;
        res
    }

    fn inc_tcp_msg_id(&mut self) -> usize {
        let res = self.next_tcp_msg_id;
        self.next_tcp_msg_id += 1;
        res
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn register_event(&mut self, event: &Event) {
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

    fn cancel_events(&mut self, pred: impl Fn(&Event) -> bool) {
        let to_cancel = self
            .unhandled_events
            .iter()
            .cloned()
            .filter(|id| pred(&self.events[*id]))
            .collect::<Vec<_>>();
        for event in to_cancel {
            let event = &self.events[event];
            self.driver
                .upgrade()
                .expect("can not upgrade driver")
                .borrow_mut()
                .cancel_event(event);
            let remove_result = self.unhandled_events.remove(&event.id);
            match &event.info {
                EventInfo::UdpMessage(msg) => {
                    let entry = UdpMessageDropped {
                        from: msg.from.address(),
                        to: msg.to.address(),
                        content: msg.content.clone(),
                        time: self.time,
                    };
                    self.event_log
                        .borrow_mut()
                        .add_entry(LogEntry::UdpMessageDropped(entry));
                }
                EventInfo::TcpMessage(msg) => {
                    let entry = TcpMessageDropped {
                        from: msg.from.address(),
                        to: msg.to.address(),
                        packet: msg.packet.clone(),
                        time: self.time,
                    };
                    self.event_log
                        .borrow_mut()
                        .add_entry(LogEntry::TcpMessageDropped(entry));
                }
                _ => {}
            }
            assert!(remove_result);
        }
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
            time: driver.borrow().start_time(),
            driver: Rc::downgrade(driver),
            next_udp_msg_id: 0,
            next_tcp_msg_id: 0,
            next_tcp_stream_id: 0,
            stat: Default::default(),
            unhandled_events: Default::default(),
            tcp: Default::default(),
            rpc: Default::default(),
            timers: Default::default(),
        };
        Self(Rc::new(RefCell::new(state)))
    }

    pub fn handle(&self) -> EventManagerHandle {
        EventManagerHandle(Rc::downgrade(&self.0))
    }

    pub fn set_system_handle(&self, handle: SystemHandle) {
        self.0.borrow_mut().system = Some(handle);
    }

    pub fn hash(&self, ctx: HashContext) -> HashType {
        self.0.borrow().hash(ctx)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct EventManagerHandle(Weak<RefCell<EventManagerState>>);

impl EventManagerHandle {
    fn state(&self) -> Rc<RefCell<EventManagerState>> {
        self.0.upgrade().expect("can not upgrade manager handle")
    }

    pub(crate) fn tcp_registry(&self) -> Rc<RefCell<dyn TcpRegistry>> {
        self.state()
    }

    pub(crate) fn rpc_registry(&self) -> Rc<RefCell<dyn RpcRegistry>> {
        self.state()
    }

    pub(crate) fn fs_registry(&self) -> Rc<RefCell<dyn FsEventRegistry>> {
        self.state()
    }

    pub(crate) fn timer_registry(&self) -> Rc<RefCell<dyn TimerRegistry>> {
        self.state()
    }

    fn guard(&self, proc: ProcessHandle) -> Guard {
        let fs = self.state().borrow().system().fs(&proc.address().node);

        let ctx = Context {
            event_manager: self.clone(),
            proc,
            fs,
        };

        Guard::new(ctx)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn time(&self) -> Time {
        self.state().borrow().time
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn log(&self) -> Log {
        self.state().borrow().event_log.borrow().clone()
    }

    pub fn cancel_events(&self, pred: impl Fn(&Event) -> bool) {
        self.state().borrow_mut().cancel_events(pred);
    }

    pub fn on_node_crash(&self, node: &str) {
        // add log entry
        let crashed_entry = NodeCrashed {
            node: node.to_string(),
            time: self.time(),
        };
        let crashed_entry = LogEntry::NodeCrashed(crashed_entry);
        self.state()
            .borrow_mut()
            .event_log
            .borrow_mut()
            .add_entry(crashed_entry);

        // cancel events with predicate
        self.cancel_events(|e| match &e.info {
            EventInfo::UdpMessage(msg) => {
                msg.from.address().node == node || msg.to.address().node == node
            }
            EventInfo::TcpMessage(msg) => {
                msg.from.address().node == node || msg.to.address().node == node
            }
            EventInfo::Timer(timer) => timer.proc.address().node == node,
            EventInfo::TcpEvent(e) => e.to.address().node == node,
            EventInfo::FsEvent(e) => e.proc.node == node,
            EventInfo::RpcEvent(e) => e.to.address().node == node,
            EventInfo::RpcMessage(msg) => {
                msg.from.address().node == node || msg.to.address().node == node
            }
        });

        self.state().borrow_mut().stat.nodes_crashed += 1;
    }

    pub fn add_log(&self, process: ProcessHandle, content: String) {
        let state = self.state();
        let state = state.borrow_mut();
        let time = state.time;
        let info = ProcessInfo {
            process: process.address(),
            time,
            content,
        };
        let entry = LogEntry::ProcessInfo(info);
        state.event_log.borrow_mut().add_entry(entry);
    }

    pub fn stat(&self) -> EventStat {
        self.state().borrow().stat.clone()
    }

    pub fn pending_events(&self) -> usize {
        self.state().borrow().unhandled_events.len()
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
            state.event_log.borrow_mut().add_entry(log_entry);
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
            state.event_log.borrow_mut().add_entry(log_entry);
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
        let (shift_min, shift_max) = net.delays_range();
        let event = Event {
            id: state.events.len(),
            time: state.time.shift_range(shift_min, shift_max),
            info,
            on_happen: None,
        };

        // register event
        state.register_event(&event);
        state.events.push(event);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_local_msg_from_process(&self, proc: ProcessHandle, content: String) {
        // log
        {
            let state = self.state();
            let state = state.borrow_mut();

            let log_entry = ProcessSentLocalMessage {
                process: proc.address(),
                content: content.clone(),
                time: state.time,
            };
            let log_entry = LogEntry::ProcessSentLocalMessage(log_entry);
            state.event_log.borrow_mut().add_entry(log_entry);
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
            let event = state.events[event_id].cloned();

            // update time

            assert!(state.time <= outcome.time);

            state.time = outcome.time;

            // return event
            event
        };
        match &outcome.kind {
            EventOutcomeKind::UdpMessageDropped() => self.handle_udp_message_dropped(&event),
            EventOutcomeKind::UdpMessageDelivered() => self.handle_udp_message_delivered(&event),
            EventOutcomeKind::TimerFired() => self.handle_timer_fired(&event),
            EventOutcomeKind::TcpPacketDelivered() => {
                let _ = event
                    .on_happen
                    .unwrap()
                    .invoke::<Result<(), TcpError>>(Ok(()));
            }
            EventOutcomeKind::TcpEventHappen(r) => {
                let _ = event.on_happen.unwrap().invoke(r.clone());
            }
            EventOutcomeKind::FsEventHappen(outcome) => {
                let _ = event.on_happen.unwrap().invoke(outcome.clone());
            }
            EventOutcomeKind::RpcMessageDelivered => {
                let _ = event.on_happen.unwrap().invoke::<RpcResult<()>>(Ok(()));
            }
            EventOutcomeKind::RpcEventHappen(r) => {
                let _ = event.on_happen.unwrap().invoke(r.clone());
            }
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
            time: state.time,
        };
        let log_entry = LogEntry::UdpMessageDropped(dropped_entry);
        state.event_log.borrow_mut().add_entry(log_entry);
        state.stat.udp_msg_dropped += 1;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn handle_udp_message_delivered(&self, event: &Event) {
        let msg = variant::variant!(&event.info, EventInfo::UdpMessage(msg));

        // add log entry
        let recv = {
            let state = self.state();
            let state = state.borrow_mut();

            let received_entry = UdpMessageReceived {
                from: msg.from.address(),
                to: msg.to.address(),
                content: msg.content.clone(),
                time: state.time,
            };
            let log_entry = LogEntry::UdpMessageReceived(received_entry);
            state.event_log.borrow_mut().add_entry(log_entry);

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

            let entry = if timer.with_sleep {
                let wakeup_entry = FutureWokeUp {
                    tag: timer.timer_id,
                    proc: timer.proc.address(),
                    time: state.time,
                };
                LogEntry::FutureWokeUp(wakeup_entry)
            } else {
                let timer_entry = TimerFired {
                    id: timer.timer_id,
                    proc: timer.proc.address(),
                    time: state.time,
                };
                LogEntry::TimerFired(timer_entry)
            };
            state.event_log.borrow_mut().add_entry(entry);

            // get sender
            state.timers.remove(timer.timer_id)
        };

        // wakeup sleeping future
        if let Some(sender) = sender {
            let _ = sender.send(());
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn handle_local_msg_from_user(&self, proc: ProcessHandle, content: String) {
        // log
        {
            let state = self.state();
            let state = state.borrow_mut();

            let log_entry = ProcessReceivedLocalMessage {
                process: proc.address(),
                content: content.clone(),
                time: state.time,
            };
            let log_entry = LogEntry::ProcessReceivedLocalMessage(log_entry);
            state.event_log.borrow_mut().add_entry(log_entry);
        }

        // store local
        let _guard = self.guard(proc.clone());
        proc.proc().borrow_mut().on_local_message(content);
    }
}

////////////////////////////////////////////////////////////////////////////////
// TCP
////////////////////////////////////////////////////////////////////////////////

impl EventManagerState {
    fn make_dummy_proc_handle() -> ProcessHandle {
        struct DummyProc {}
        impl Process for DummyProc {
            fn on_message(&mut self, _from: Address, _content: String) {
                unreachable!()
            }

            fn on_local_message(&mut self, _content: String) {
                unreachable!()
            }

            fn hash(&self) -> HashType {
                unreachable!()
            }
        }

        let proc = Rc::new(RefCell::new(DummyProc {}));
        let dummy_proc = Rc::new(RefCell::new(ProcessState::new(proc, "0:0".into())));
        ProcessHandle::new(&dummy_proc)
    }

    fn make_and_register_tcp_event(
        &mut self,
        kind: TcpEventKind,
        to: ProcessHandle,
        trigger: Trigger,
    ) -> &Event {
        let event = TcpEvent { kind, to };
        let (min_shift, max_shift) = self.system().network().delays_range();
        let event = Event {
            id: self.events.len(),
            time: self.time.shift_range(min_shift, max_shift),
            info: EventInfo::TcpEvent(event),
            on_happen: Some(trigger),
        };

        self.register_event(&event);
        self.events.push(event);

        self.events.last().unwrap()
    }

    fn make_and_register_tcp_message(
        &mut self,
        from: ProcessHandle,
        to: ProcessHandle,
        packet: &TcpPacket,
        trigger: Trigger,
    ) -> &Event {
        let msg = TcpMessage {
            tcp_msg_id: self.inc_tcp_msg_id(),
            from,
            to,
            packet: packet.clone(),
        };
        let (min_shift, max_shift) = self.system().network().delays_range();
        let event = Event {
            id: self.events.len(),
            time: self.time.shift_range(min_shift, max_shift),
            info: EventInfo::TcpMessage(msg),
            on_happen: Some(trigger),
        };
        self.register_event(&event);
        self.events.push(event);
        self.events.last().unwrap()
    }
}

impl TcpRegistry for EventManagerState {
    fn emit_packet(
        &mut self,
        from: &Address,
        to: &Address,
        packet: &TcpPacket,
        on_delivery: Trigger,
    ) -> Result<(), TcpError> {
        // add log entry
        {
            let log_entry = TcpMessageSent {
                from: from.clone(),
                to: to.clone(),
                packet: packet.clone(),
                time: self.time,
            };
            let log_entry = LogEntry::TcpMessageSent(log_entry);
            self.event_log.borrow_mut().add_entry(log_entry);
        }

        let from_proc = self.system().proc_by_addr(from).unwrap();
        let to_proc = self.system().proc_by_addr(to);
        let to_proc = if let Some(to_proc) = to_proc {
            to_proc
        } else {
            // add dropped entry
            let log_entry = TcpMessageDropped {
                from: from_proc.address(),
                to: to.clone(),
                packet: packet.clone(),
                time: self.time,
            };
            let log_entry = LogEntry::TcpMessageDropped(log_entry);
            self.event_log.borrow_mut().add_entry(log_entry);

            // schedule event
            self.make_and_register_tcp_event(
                TcpEventKind::ConnectionRefused,
                from_proc,
                on_delivery,
            );

            return Ok(());
        };

        self.make_and_register_tcp_message(from_proc, to_proc, packet, on_delivery);
        Ok(())
    }

    fn emit_listen_request(&mut self, from: &Address, on_listen: Trigger) -> Result<(), TcpError> {
        self.tcp.listen(from, on_listen)
    }

    fn emit_listen_to_request(
        &mut self,
        from: &Address,
        to: &Address,
        on_listen: Trigger,
    ) -> Result<(), TcpError> {
        self.tcp.listen_to(from, to, on_listen)
    }

    fn emit_sender_dropped(&mut self, sender: &mut TcpSender) {
        if self.system().system_dropped() {
            return;
        }

        let (waiter, trigger) = make_trigger();
        let Some(to) = self.system().proc_by_addr(&sender.other) else {
            return;
        };

        self.make_and_register_tcp_event(TcpEventKind::SenderDropped, to, trigger);

        let sender = sender.sender.clone();

        println!("spawning...");

        self.rt.spawn(
            async move {
                println!("sender dropped, register waiter");
                let _ = waiter.wait::<Result<(), TcpError>>().await;
                println!("waiter.wait done");
                drop(sender);
            },
            Self::make_dummy_proc_handle(),
        );
    }

    fn register_packet_delivery(
        &mut self,
        from: &Address,
        to: &Address,
        packet: &TcpPacket,
    ) -> Result<(), TcpError> {
        let log_entry = TcpMessageReceived {
            from: from.clone(),
            to: to.clone(),
            packet: packet.clone(),
            time: self.time,
        };
        let log_entry = LogEntry::TcpMessageReceived(log_entry);
        self.event_log.borrow_mut().add_entry(log_entry);
        Ok(())
    }

    fn try_connect(
        &mut self,
        from: &Address,
        to: &Address,
        stream_id: usize,
        registry_ref: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        self.tcp.connect(from, to, stream_id, registry_ref)
    }

    fn next_tcp_stream_id(&mut self) -> usize {
        let res = self.next_tcp_stream_id;
        self.next_tcp_stream_id += 1;
        res
    }
}

////////////////////////////////////////////////////////////////////////////////
// FS
////////////////////////////////////////////////////////////////////////////////

impl FsEventRegistry for EventManagerState {
    fn register_instant_event(&mut self, event: &FsEvent) {
        let entry = event.clone().make_log_entry_on_init(self.time);
        self.event_log.borrow_mut().add_entry(entry);
    }

    fn register_event_initiated(&mut self, event: &FsEvent) {
        let entry = event.clone().make_log_entry_on_init(self.time);
        self.event_log.borrow_mut().add_entry(entry);
    }

    fn register_event_pipelined(&mut self, trigger: Trigger, event: &FsEvent) {
        // no log here
        let e = super::info::FsEvent {
            proc: event.initiated_by.clone(),
            kind: event.kind.clone(),
            outcome: event.outcome.clone(),
        };
        let info = EventInfo::FsEvent(e);
        let event = Event {
            id: self.events.len(),
            time: self.time.shift_on(event.delay),
            info,
            on_happen: Some(trigger),
        };
        self.register_event(&event);
        self.events.push(event);
    }

    fn register_event_happen(&mut self, event: &FsEvent) {
        let entry = event.clone().make_log_entry_on_complete(self.time);
        self.event_log.borrow_mut().add_entry(entry);
    }
}

////////////////////////////////////////////////////////////////////////////////
// RPC
////////////////////////////////////////////////////////////////////////////////

impl RpcRegistry for EventManagerState {
    fn next_request_id(&mut self) -> u64 {
        self.rpc.borrow_mut().inc_next_id()
    }

    fn register_request(
        &mut self,
        request: RpcRequest,
    ) -> oneshot::Receiver<RpcResult<RpcResponse>> {
        // add log entry
        {
            let entry = RpcMessageSent {
                from: request.from.clone(),
                to: request.to.clone(),
                content: request.content.clone(),
                time: self.time,
            };
            self.event_log
                .borrow_mut()
                .add_entry(LogEntry::RpcMessageSent(entry));
        }
        let (min_net_delay, max_net_delay) = self.system().network().delays_range();
        let from_proc = self.system().proc_by_addr(&request.from).unwrap();
        let has_listener = self.rpc.borrow().has_listener(&request.to);
        let (waiter, trigger) = trigger::make_trigger();
        let event = if !has_listener {
            let event = RpcEvent {
                kind: RpcEventKind::ConnectionRefused,
                to: from_proc,
            };
            Event {
                id: self.events.len(),
                time: self.time.shift_range(min_net_delay, max_net_delay),
                info: EventInfo::RpcEvent(event),
                on_happen: Some(trigger),
            }
        } else {
            let to_proc = self.system().proc_by_addr(&request.to).unwrap();
            let event = RpcMessage {
                from: from_proc,
                to: to_proc,
                kind: RpcMessageKind::Request {
                    id: request.id,
                    tag: request.tag,
                    content: request.content.clone(),
                },
            };
            Event {
                id: self.events.len(),
                time: self.time.shift_range(min_net_delay, max_net_delay),
                info: EventInfo::RpcMessage(event),
                on_happen: Some(trigger),
            }
        };

        self.register_event(&event);
        self.events.push(event);

        let rpc = self.rpc.clone();
        let (sender, receiver) = oneshot::channel();

        let log = self.event_log.clone();

        self.rt.spawn(
            async move {
                // wait for event to happen
                let result = waiter.wait::<RpcResult<()>>().await.unwrap();

                // send on failure
                if let Err(e) = result {
                    // add log entry
                    {
                        let entry = RpcMessageDropped {
                            from: request.from.clone(),
                            to: request.to.clone(),
                            content: request.content.clone(),
                            time: time(),
                        };
                        log.borrow_mut()
                            .add_entry(LogEntry::RpcMessageDropped(entry));
                    }
                    let _ = sender.send(Err(e));
                    return;
                }

                let from = request.from.clone();
                let to = request.to.clone();
                let content = request.content.clone();

                // register sent request
                let result = rpc.borrow_mut().send_request(request);
                let result = match result {
                    Ok(r) => {
                        // add log entry
                        {
                            let entry = RpcMessageReceived {
                                from,
                                to,
                                content,
                                time: time(),
                            };
                            log.borrow_mut()
                                .add_entry(LogEntry::RpcMessageReceived(entry));
                        }

                        r.await.unwrap()
                    }
                    Err(e) => {
                        // add log entry
                        {
                            let entry = RpcMessageDropped {
                                from,
                                to,
                                content,
                                time: time(),
                            };
                            log.borrow_mut()
                                .add_entry(LogEntry::RpcMessageDropped(entry));
                        }

                        Err(e)
                    }
                };

                // send
                let _ = sender.send(result);
            },
            Self::make_dummy_proc_handle(),
        );

        receiver
    }

    fn register_response(
        &mut self,
        from: Address,
        to: Address,
        request_id: u64,
        response: RpcResult<RpcResponse>,
    ) -> RpcResult<()> {
        if self.system().system_dropped() {
            return Ok(());
        }

        // add log entry
        let content = match &response {
            Ok(e) => e.content.clone(),
            Err(e) => e.to_string().into_bytes(),
        };
        {
            let entry = RpcMessageSent {
                from: from.clone(),
                to: to.clone(),
                content: content.clone(),
                time: self.time,
            };
            self.event_log
                .borrow_mut()
                .add_entry(LogEntry::RpcMessageSent(entry));
        };

        let receiver_alive = self.rpc.borrow().response_receiver_alive(request_id);
        if !receiver_alive {
            return Ok(());
        }

        let to_proc = self.system().proc_by_addr(&to).unwrap();

        let from_proc = self.system().proc_by_addr(&from);

        let event = if let Some(from_proc) = from_proc {
            let msg = RpcMessage {
                from: from_proc,
                to: to_proc,
                kind: RpcMessageKind::Response {
                    id: request_id,
                    content: match &response {
                        Ok(e) => Ok(e.content.clone()),
                        Err(e) => Err(e.clone()),
                    },
                },
            };
            EventInfo::RpcMessage(msg)
        } else {
            let e = RpcEvent {
                kind: RpcEventKind::ConnectionRefused,
                to: to_proc,
            };
            EventInfo::RpcEvent(e)
        };

        let (min_net_delay, max_net_delay) = self.system().network().delays_range();

        let (waiter, trigger) = make_trigger();

        let event = Event {
            id: self.events.len(),
            time: self.time.shift_range(min_net_delay, max_net_delay),
            info: event,
            on_happen: Some(trigger),
        };

        self.register_event(&event);
        self.events.push(event);

        let rpc = self.rpc.clone();
        let log = self.event_log.clone();
        self.rt.spawn(
            async move {
                let result = waiter.wait::<RpcResult<()>>().await.unwrap();
                let entry = match result {
                    Err(e) => {
                        let _ = rpc.borrow_mut().send_response(request_id, Err(e.clone()));

                        let entry = RpcMessageDropped {
                            from,
                            to,
                            content,
                            time: time(),
                        };
                        LogEntry::RpcMessageDropped(entry)
                    }
                    Ok(_) => {
                        let _ = rpc.borrow_mut().send_response(request_id, response);

                        let entry = RpcMessageReceived {
                            from,
                            to,
                            content,
                            time: time(),
                        };
                        LogEntry::RpcMessageReceived(entry)
                    }
                };
                log.borrow_mut().add_entry(entry);
            },
            Self::make_dummy_proc_handle(),
        );

        Ok(())
    }

    fn register_listener(&mut self, from: Address) -> RpcResult<RpcListener> {
        self.rpc.borrow_mut().register_listener(from)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Timer
////////////////////////////////////////////////////////////////////////////////

impl TimerRegistry for EventManagerState {
    fn register_timer(
        &mut self,
        duration: Duration,
        with_sleep: bool,
        proc: Address,
    ) -> (usize, oneshot::Receiver<()>) {
        let time = self.time.shift(duration);
        let id = self.events.len();
        let receiver = self.timers.create(id);
        let proc_handle = self.system().proc_by_addr(&proc).unwrap();

        // register log entry
        {
            let entry = if !with_sleep {
                let entry = TimerSet {
                    id,
                    proc,
                    time,
                    duration,
                };
                LogEntry::TimerSet(entry)
            } else {
                let entry = FutureFellAsleep {
                    tag: id,
                    proc,
                    time,
                    duration,
                };
                LogEntry::FutureFellAsleep(entry)
            };
            self.event_log.borrow_mut().add_entry(entry);
        }

        let event = {
            let info = EventInfo::Timer(Timer {
                timer_id: id,
                with_sleep,
                proc: proc_handle,
                duration,
            });
            Event {
                id,
                time,
                info,
                on_happen: None,
            }
        };
        self.register_event(&event);
        self.events.push(event);

        (id, receiver)
    }

    fn cancel_timer(&mut self, id: usize, proc: Address) {
        println!("cancel timer id={id}");
        let timer = self.timers.remove(id);
        if timer.is_some() {
            let entry = TimerCanceled {
                id,
                proc,
                time: self.time,
            };
            let entry = LogEntry::TimerCanceled(entry);
            self.event_log.borrow_mut().add_entry(entry);

            self.cancel_events(|e| {
                variant::try_variant!(e.info, EventInfo::Timer(Timer { timer_id, .. }))
                    .map(|tid| tid == id)
                    .unwrap_or(false)
            });
        }
    }
}
