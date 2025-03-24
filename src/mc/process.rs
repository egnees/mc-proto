use std::future::Future;

use crate::rt;

use super::{
    message::Message,
    state::{self, Handle},
};

////////////////////////////////////////////////////////////////////////////////

pub type ProcessId = usize;

////////////////////////////////////////////////////////////////////////////////

pub trait Process {
    fn on_message(&mut self, from: ProcessId, message: Message);

    fn on_local_message(&mut self, message: Message);
}

////////////////////////////////////////////////////////////////////////////////

pub struct ProcessInfo {
    pub id: ProcessId,
    pub sent_messages: usize,
    pub received_messages: usize,
    pub pending_local: Vec<Message>,
}

impl ProcessInfo {
    pub fn from_id(id: ProcessId) -> Self {
        Self {
            id,
            sent_messages: 0,
            received_messages: 0,
            pending_local: Vec::new(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub type CreateProcessFn = Box<dyn Fn() -> Box<dyn Process>>;

////////////////////////////////////////////////////////////////////////////////

pub fn spawn<F>(task: F) -> rt::JoinHandle<F::Output>
where
    F: Future + 'static,
{
    Handle::current().spawn(task)
}

////////////////////////////////////////////////////////////////////////////////

pub fn sleep(duration: f64) -> rt::JoinHandle<()> {
    Handle::current().sleep(duration)
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_message(receiver: ProcessId, message: Message) {
    Handle::current().send_message(receiver, message)
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_local(message: Message) {
    Handle::current().send_local(message)
}
