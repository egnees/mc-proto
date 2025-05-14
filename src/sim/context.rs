use std::{cell::RefCell, future::Future};

use crate::{
    event::manager::EventManagerHandle, fs::manager::FsManagerHandle, runtime::JoinHandle,
};

use super::proc::{Address, ProcessHandle};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Context {
    pub event_manager: EventManagerHandle,
    pub proc: ProcessHandle,
    pub fs: Option<FsManagerHandle>,
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

    ////////////////////////////////////////////////////////////////////////////////

    pub fn register_udp_message(&self, to: &Address, content: String) {
        self.event_manager
            .register_udp_message(self.proc.clone(), to, content);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn send_local(&self, content: String) {
        self.event_manager
            .register_local_msg_from_process(self.proc.clone(), content);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        self.event_manager
            .register_async_task(task, self.proc.clone())
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
