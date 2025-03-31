use std::{future::Future, time::Duration};

use crate::runtime::JoinHandle;

use super::{context::Context, sys::HashType};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Hash, PartialEq)]
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

////////////////////////////////////////////////////////////////////////////////

pub trait Process {
    fn on_message(&mut self, from: Address, content: String);

    fn on_local_message(&mut self, content: String);

    fn hash(&self) -> HashType;
}

////////////////////////////////////////////////////////////////////////////////

pub fn send_local(content: impl Into<String>) {
    Context::current().send_local(content.into());
}

////////////////////////////////////////////////////////////////////////////////

pub async fn sleep(duration: Duration) {
    let recv = Context::current().register_sleep(duration);
    recv.await.unwrap();
}

////////////////////////////////////////////////////////////////////////////////

pub fn spawn<F>(task: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
{
    Context::current().spawn(task)
}
