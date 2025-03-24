use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    rc::{Rc, Weak},
};

use tokio::sync::oneshot;

use crate::{
    mc::event::{TimerFiredEvent, TimerSetEvent},
    rt::{self, Runtime},
};

use super::{
    event::{
        Event, EventId, LocalMessageReceivedEvent, LocalMessageSentEvent, MessageDroppedEvent,
        MessageReceivedEvent, MessageSentEvent, PendingEvents,
    },
    log::Log,
    message::{Message, MessageId},
    process::{CreateProcessFn, Process, ProcessId, ProcessInfo},
    time::{TimerId, TimerInfo},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct State {
    pending_events: PendingEvents,
    pub process_infos: Vec<ProcessInfo>,
    timers: HashMap<TimerId, TimerInfo>,
    messages: HashMap<MessageId, MessageReceivedEvent>,
    next_event_id: usize,
    pub log: Log,
    pub timers_set: usize,
    pub timers_fired: usize,
    pub mesages_sent: usize,
    pub messages_received: usize,
    pub messages_dropped: usize,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Handle {
    state: Weak<RefCell<State>>,
    current_process: ProcessId,
}

impl Handle {
    fn new(state: Rc<RefCell<State>>, current_process: ProcessId) -> Self {
        Self {
            state: Rc::downgrade(&state),
            current_process,
        }
    }

    fn state(&self) -> Rc<RefCell<State>> {
        self.state.upgrade().unwrap()
    }

    pub fn spawn<F>(&self, task: F) -> rt::JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        rt::spawn(task)
    }

    pub fn sleep(&self, _duration: f64) -> rt::JoinHandle<()> {
        let binding = self.state();
        let mut state = binding.borrow_mut();

        let event_id = state.next_event_id;
        state.next_event_id += 1;
        let event = TimerSetEvent {
            timer_id: event_id,
            proc: self.current_process,
        };
        state.log.0.push(Event::TimerSet(event));
        state.pending_events.add(EventId::TimerFired(event_id));

        let (waker, waiter) = oneshot::channel();
        let timer_info = TimerInfo {
            id: event_id,
            waker,
            event: TimerFiredEvent {
                timer_id: event_id,
                proc: self.current_process,
            },
        };
        state.timers.insert(event_id, timer_info);
        state.timers_set += 1;

        rt::spawn(async move {
            let result = waiter.await.unwrap(); // does not support cancel timers currently.
            assert_eq!(result, true);
        })
    }

    pub fn send_message(&self, receiver: ProcessId, message: Message) {
        let binding = self.state();
        let mut state = binding.borrow_mut();

        let event_id = state.next_event_id;
        state.next_event_id += 1;
        let event = MessageSentEvent {
            message_id: event_id,
            sender: self.current_process,
            receiver,
            content: message.clone(),
        };
        state.log.0.push(Event::MessageSent(event));
        state.pending_events.add(EventId::MessageReceived(event_id));
        let event = MessageReceivedEvent {
            message_id: event_id,
            sender: self.current_process,
            receiver,
            content: message.clone(),
        };
        state.messages.insert(event_id, event);
        state.process_infos[self.current_process].sent_messages += 1;
        state.mesages_sent += 1;
    }

    pub fn send_local(&self, message: Message) {
        let binding = self.state();
        let mut state = binding.borrow_mut();

        let event_id = state.next_event_id;
        state.next_event_id += 1;
        let event = LocalMessageSentEvent {
            message_id: event_id,
            process: self.current_process,
            content: message.clone(),
        };
        state.log.0.push(Event::LocalMessageSent(event));
        state.process_infos[self.current_process]
            .pending_local
            .push(message);
    }

    pub fn current() -> Self {
        HANDLE.with(|h| {
            h.borrow()
                .as_ref()
                .expect("must be called in process")
                .clone()
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

thread_local! {
    static HANDLE: RefCell<Option<Handle>> = RefCell::new(None);
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct System {
    processes: Vec<Box<dyn Process>>,
    state: Rc<RefCell<State>>,
    runtime: Runtime,
}

impl System {
    pub fn from_processes(create: &[CreateProcessFn]) -> Self {
        let mut sys = Self::default();

        for (proc_id, f) in create.iter().enumerate() {
            let proc = f();
            sys.processes.push(proc);
            sys.state
                .borrow_mut()
                .process_infos
                .push(ProcessInfo::from_id(proc_id));
        }

        sys
    }

    pub fn state(&self) -> Rc<RefCell<State>> {
        self.state.clone()
    }

    pub fn from_trace_and_proc(
        trace: &[(usize, bool)],
        create: &[CreateProcessFn],
        init: &mut dyn FnMut(&mut System),
    ) -> Self {
        let mut sys = Self::from_processes(create);
        init(&mut sys);
        trace.iter().for_each(|x| {
            sys.apply_pending_event(x.0, x.1);
        });
        sys
    }

    pub fn apply_pending_event(&mut self, event_num: usize, drop: bool) -> bool {
        let mut result = false;
        let event_id = self.state.borrow_mut().pending_events.take_event(event_num);
        match event_id {
            EventId::TimerFired(timer_id) => {
                let timer_info = self.state.borrow_mut().timers.remove(&timer_id).unwrap();
                self.install_handle(timer_info.event.proc); // install
                self.state
                    .borrow_mut()
                    .log
                    .0
                    .push(Event::TimerFired(timer_info.event));
                let _ = timer_info.waker.send(true); // receiver can be dropped which is ok
                self.state.borrow_mut().timers_fired += 1;
            }
            EventId::MessageReceived(msg_id) => {
                let msg_info = self.state.borrow_mut().messages.remove(&msg_id).unwrap();
                if !drop {
                    let receiver = msg_info.receiver;
                    let sender = msg_info.sender;
                    let content = msg_info.content.clone();
                    self.state
                        .borrow_mut()
                        .log
                        .0
                        .push(Event::MessageReceived(msg_info));
                    self.state.borrow_mut().process_infos[receiver].received_messages += 1;
                    self.install_handle(receiver); // install
                    self.processes[receiver].on_message(sender, content);
                    result = true;
                    self.state.borrow_mut().messages_received += 1;
                } else {
                    let msg_dropped = MessageDroppedEvent {
                        message_id: msg_info.message_id,
                        sender: msg_info.sender,
                        receiver: msg_info.receiver,
                        content: msg_info.content,
                    };
                    self.state
                        .borrow_mut()
                        .log
                        .0
                        .push(Event::MessageDropped(msg_dropped));
                    self.state.borrow_mut().messages_dropped += 1;
                }
            }
        }
        self.runtime.process_tasks();
        self.remove_handle(); // remove
        result
    }

    pub fn pending_events_count(&self) -> usize {
        self.state.borrow().pending_events.events_count()
    }

    pub fn send_local(&mut self, proc_id: ProcessId, message: Message) {
        let event_id = {
            let mut state = self.state.borrow_mut();
            let id = state.next_event_id;
            state.next_event_id += 1;
            id
        };
        let event = LocalMessageReceivedEvent {
            message_id: event_id,
            process: proc_id,
            content: message.clone(),
        };
        self.state
            .borrow_mut()
            .log
            .0
            .push(Event::LocalMessageReceived(event));

        self.install_handle(proc_id);
        {
            self.processes[proc_id].on_local_message(message);
            self.runtime.process_tasks();
        }
        self.remove_handle();
    }

    pub fn drain_local(&mut self, proc_id: ProcessId) -> Vec<Message> {
        self.state.borrow_mut().process_infos[proc_id]
            .pending_local
            .drain(..)
            .collect()
    }

    fn install_handle(&self, current_process: ProcessId) {
        HANDLE.with(|h| *h.borrow_mut() = Some(Handle::new(self.state.clone(), current_process)));
        self.runtime.set_current_handle();
    }

    fn remove_handle(&self) {
        HANDLE.with(|h| *h.borrow_mut() = None);
        self.runtime.remove_current_handle();
    }
}
