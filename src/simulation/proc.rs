use std::{
    cell::RefCell,
    fmt::Display,
    future::Future,
    hash::Hash,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::runtime::JoinHandle;

use super::{context::Context, system::HashType};

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

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.node, self.process)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Process {
    fn on_message(&mut self, from: Address, content: String);

    fn on_local_message(&mut self, content: String);

    fn hash(&self) -> HashType;
}

////////////////////////////////////////////////////////////////////////////////

pub struct ProcessState {
    /// Process implementation.
    pub(crate) proc: Rc<RefCell<dyn Process>>,

    /// List of locals messages sent by process.
    pub(crate) locals: Vec<String>,

    /// Process address.
    pub(crate) address: Address,
}

impl ProcessState {
    pub fn new(proc: Rc<RefCell<dyn Process>>, address: Address) -> Self {
        Self {
            proc,
            locals: Vec::new(),
            address,
        }
    }

    pub fn proc(&self) -> Rc<RefCell<dyn Process>> {
        self.proc.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ProcessHandle(Weak<RefCell<ProcessState>>);

impl ProcessHandle {
    pub fn address(&self) -> Address {
        self.state().borrow().address.clone()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn new(proc: &Rc<RefCell<ProcessState>>) -> Self {
        Self(Rc::downgrade(proc))
    }

    pub(crate) fn state(&self) -> Rc<RefCell<ProcessState>> {
        self.0
            .upgrade()
            .expect("can not upgrade process handle to process state")
    }

    pub(crate) fn store_local(&self, content: String) {
        self.state().borrow_mut().locals.push(content);
    }

    pub(crate) fn read_locals(&self) -> Vec<String> {
        self.state().borrow().locals.to_vec()
    }

    pub(crate) fn drain_locals(&self) -> Vec<String> {
        self.state().borrow_mut().locals.drain(..).collect()
    }

    pub(crate) fn proc(&self) -> Rc<RefCell<dyn Process>> {
        self.state().borrow().proc()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Hash for ProcessState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.proc.borrow().hash().hash(state);
    }
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
