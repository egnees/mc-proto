use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{Address, Process};

use super::{
    context::{Context, Guard},
    node::RealNodeHandle,
};

////////////////////////////////////////////////////////////////////////////////

pub struct ProcessState {
    proc: Rc<RefCell<dyn Process>>,
    pub(crate) rpc: bool,
    name: String,
    local_sender: UnboundedSender<String>,
}

impl ProcessState {
    pub fn new(proc: impl Process, name: String, local_sender: UnboundedSender<String>) -> Self {
        Self {
            proc: Rc::new(RefCell::new(proc)),
            rpc: false,
            name,
            local_sender,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ProcessHandle {
    pub proc: Weak<RefCell<ProcessState>>,
    pub node: RealNodeHandle,
}

impl ProcessHandle {
    pub fn send_local(&self, msg: impl Into<String>) {
        if let Some(proc) = self.proc.upgrade() {
            if !Context::installed() {
                let _guard = Guard::new(Context::new(self.clone()));
                let proc = proc.borrow().proc.clone();
                proc.borrow_mut().on_local_message(msg.into());
            } else {
                let proc = proc.borrow().proc.clone();
                proc.borrow_mut().on_local_message(msg.into());
            }
        }
    }

    pub(crate) fn send_local_to_user(&self, msg: impl Into<String>) {
        let _ = self
            .proc
            .upgrade()
            .unwrap()
            .borrow()
            .local_sender
            .send(msg.into());
    }

    pub fn name(&self) -> String {
        self.proc.upgrade().unwrap().borrow().name.clone()
    }

    pub fn address(&self) -> Address {
        let node = self.node.name();
        let proc = self.name();
        Address::new(node, proc)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to send local messages to process.
/// Returned on adding process to the system (see [crate::real::RealNode::add_proc]).
pub struct LocalSender {
    pub(crate) handle: ProcessHandle,
}

impl LocalSender {
    /// Send local message to process
    pub fn send(&self, msg: impl Into<String>) {
        self.handle.send_local(msg);
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to receive local messages from process.
/// Returned on adding process to the system (see [crate::real::RealNode::add_proc]).
pub struct LocalReceiver {
    pub(crate) receiver: UnboundedReceiver<String>,
}

impl LocalReceiver {
    /// Block until local message is not received from process.
    pub async fn recv<T>(&mut self) -> Option<T>
    where
        T: From<String>,
    {
        self.receiver.recv().await.map(T::from)
    }

    /// Block current thread until local message is not received.
    pub fn blocking_recv<T>(&mut self) -> Option<T>
    where
        T: From<String>,
    {
        self.receiver.blocking_recv().map(T::from)
    }
}
