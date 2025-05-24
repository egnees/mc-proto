use std::{any::Any, fmt::Display, future::Future, time::Duration};

use crate::{real, sim, timer, HashType, Timer};

use super::{
    mode::{is_real, is_sim},
    JoinHandle,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    pub node: String,
    pub process: String,
}

impl Address {
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

pub trait Process: Any {
    fn on_message(&mut self, from: Address, content: String);

    fn on_local_message(&mut self, content: String);

    fn hash(&self) -> HashType;
}

////////////////////////////////////////////////////////////////////////////////

pub fn spawn<F>(f: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: Send,
{
    if is_real() {
        let h = real::context::Context::current().spawn(f);
        JoinHandle::Real(h)
    } else {
        let h = sim::context::Context::current().spawn(f);
        JoinHandle::Sim(h)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_local(msg: impl Into<String>) {
    if is_real() {
        real::context::Context::current().send_local(msg.into());
    } else {
        sim::context::Context::current().send_local(msg.into());
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn log(content: impl Into<String>) {
    if is_sim() {
        let context = sim::context::Context::current();
        let proc = context.proc;
        context.event_manager.add_log(proc, content.into());
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn set_timer(duration: Duration) -> Timer {
    if is_sim() {
        let timer = timer::set_timer(duration);
        Timer::Sim(timer)
    } else {
        let timer = real::context::Context::current().set_timer(duration);
        Timer::Real(timer)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn set_random_timer(min_duration: Duration, max_duration: Duration) -> Timer {
    if is_sim() {
        let timer = timer::set_random_timer(min_duration, max_duration);
        Timer::Sim(timer)
    } else {
        let timer = real::context::Context::current().set_random_timer(min_duration, max_duration);
        Timer::Real(timer)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub async fn sleep(duration: Duration) {
    set_timer(duration).await;
}
