use std::{
    cell::RefCell,
    future::Future,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{
    event::info::EventInfo,
    runtime::{JoinHandle, RuntimeHandle},
    util::oneshot,
};

use super::{
    log::{FutureFellAsleep, LogEntry, ProcessSentLocalMessage, UdpMessageSent},
    proc::Address,
    sys::State,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Context {
    pub state: Weak<RefCell<State>>,
    pub cur_proc: Address,
    pub rt: RuntimeHandle,
}

////////////////////////////////////////////////////////////////////////////////

impl Context {
    pub fn current() -> Context {
        CONTEXT.with(|c| {
            c.borrow()
                .as_ref()
                .expect("context is not installed")
                .clone()
        })
    }

    fn install(ctx: Context) {
        CONTEXT.with(|c| *c.borrow_mut() = Some(ctx));
    }

    fn reset() {
        CONTEXT.with(|c| *c.borrow_mut() = None);
    }

    fn state(&self) -> Rc<RefCell<State>> {
        self.state
            .upgrade()
            .expect("can not upgrade weak ptr on state")
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_udp_message(&self, to: &Address, content: String) {
        let state = self.state();
        let mut state = state.borrow_mut();

        // add log entry
        {
            let log_entry = UdpMessageSent {
                from: self.cur_proc.clone(),
                to: to.clone(),
                content: content.to_string(),
            };
            let log_entry = LogEntry::UdpMessageSent(log_entry);
            state.log.add_entry(log_entry);
        }

        // register udp message
        let time_from = state.time_from + state.net.min_packet_delay;
        let time_to = state.time_to + state.net.max_packet_delay;
        state.events.register_udp_message(
            self.cur_proc.clone(),
            to.clone(),
            content,
            time_from,
            time_to,
        );
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_sleep(&self, duration: Duration) -> oneshot::Receiver<bool> {
        let state = self.state();
        let mut state = state.borrow_mut();

        // register timer
        let time_from = state.time_from + duration;
        let time_to = state.time_to + duration;
        let event = state
            .events
            .register_timer(self.cur_proc.clone(), true, time_from, time_to);
        let timer_id = if let EventInfo::TimerInfo(timer_info) = &event.info {
            timer_info.timer_id
        } else {
            unreachable!()
        };
        let (rx, tx) = oneshot::channel();
        state.timers.insert(timer_id, rx);

        // add log entry
        {
            let log_entry = FutureFellAsleep {
                tag: timer_id,
                proc: self.cur_proc.clone(),
                duration,
            };
            let log_entry = LogEntry::FutureFellAsleep(log_entry);
            state.log.add_entry(log_entry);
        }

        tx
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn send_local(&self, content: String) {
        let state = self.state();
        let mut state = state.borrow_mut();

        // add log entry
        {
            let log_entry = ProcessSentLocalMessage {
                process: self.cur_proc.clone(),
                content: content.clone(),
            };
            let log_entry = LogEntry::ProcessSentLocalMessage(log_entry);
            state.log.add_entry(log_entry);
        }

        // send
        state
            .nodes
            .get_mut(&self.cur_proc.node)
            .unwrap()
            .get_locals(&self.cur_proc.process)
            .unwrap()
            .push(content);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        let handle = self.rt.spawn(task);

        self.state()
            .borrow_mut()
            .async_proc
            .insert(handle.id(), self.cur_proc.clone());

        handle
    }
}

////////////////////////////////////////////////////////////////////////////////

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

////////////////////////////////////////////////////////////////////////////////

pub struct Guard {}

impl Guard {
    pub fn new(ctx: Context) -> Guard {
        CONTEXT.with(|c| assert!(c.borrow().is_none()));
        Context::install(ctx);
        Guard {}
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        Context::reset();
    }
}
