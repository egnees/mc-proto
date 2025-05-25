//! Represents process abstraction.

use std::{any::Any, fmt::Display, future::Future, time::Duration};

use crate::{
    model::{self, timer},
    real, HashType, Timer,
};

use super::{
    mode::{is_real, is_sim},
    JoinHandle,
};

////////////////////////////////////////////////////////////////////////////////

/// Represents address of the process
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    /// Name of the process node
    pub node: String,

    /// Name of the process
    pub process: String,
}

impl Address {
    /// Create new address from node name and process name.
    pub fn new(node: impl Into<String>, process: impl Into<String>) -> Self {
        Self {
            node: node.into(),
            process: process.into(),
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.node, self.process)
    }
}

impl<T> From<T> for Address
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let s: String = value.into();
        let pos = s.find(":").expect("can not find division symbol ':'");
        let (node, proc) = s.split_at(pos);
        Address::new(node, &proc[1..])
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents process trait.
pub trait Process: Any {
    /// Called when process receives message.
    fn on_message(&mut self, from: Address, content: String);

    /// Called when process receives local message.
    fn on_local_message(&mut self, content: String);

    /// Get hash of the process state.
    fn hash(&self) -> HashType;
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to spawn async activity.
pub fn spawn<F>(f: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: Send,
{
    if is_real() {
        let h = real::context::Context::current().spawn(f);
        JoinHandle::Real(h)
    } else {
        let h = model::context::Context::current().spawn(f);
        JoinHandle::Model(h)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allwos to send local message to user.
pub fn send_local(msg: impl Into<String>) {
    if is_real() {
        real::context::Context::current().send_local(msg.into());
    } else {
        model::context::Context::current().send_local(msg.into());
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to log some message from the process.
pub fn log(content: impl Into<String>) {
    if is_sim() {
        let context = model::context::Context::current();
        let proc = context.proc;
        context.event_manager.add_log(proc, content.into());
    } else if is_real() {
        let proc = real::context::Context::current().proc_addr();
        println!("{proc} === {:?}", content.into());
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to set time with specified duration.
pub fn set_timer(duration: Duration) -> Timer {
    if is_sim() {
        let timer = timer::set_timer(duration);
        Timer::Model(timer)
    } else {
        let timer = real::context::Context::current().set_timer(duration);
        Timer::Real(timer)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to set timer with random duration from specified range.
///
/// In the MC [`crate::mc`], timer is handled as event, which can happen in any moment from
/// the specified range.
pub fn set_random_timer(min_duration: Duration, max_duration: Duration) -> Timer {
    if is_sim() {
        let timer = timer::set_random_timer(min_duration, max_duration);
        Timer::Model(timer)
    } else {
        let timer = real::context::Context::current().set_random_timer(min_duration, max_duration);
        Timer::Real(timer)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to sleep on the provided time.
pub async fn sleep(duration: Duration) {
    set_timer(duration).await;
}
