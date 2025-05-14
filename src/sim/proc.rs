use std::{
    any::{Any, TypeId},
    cell::RefCell,
    fmt::Display,
    future::Future,
    hash::Hash,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{event::time::Time, runtime::JoinHandle, timer};

use super::{context::Context, system::HashType};

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

    pub fn get_as<T: Any>(&self) -> Option<Rc<RefCell<T>>> {
        let proc = self.proc();
        let tid = {
            let proc = proc.clone() as Rc<RefCell<dyn Any>>;
            let b = proc.borrow();
            (*b).type_id()
        };
        if TypeId::of::<T>() == tid {
            let result = unsafe { Rc::from_raw(Rc::into_raw(proc) as *const RefCell<T>) };
            Some(result)
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ProcessHandle(Weak<RefCell<ProcessState>>);

impl ProcessHandle {
    pub fn try_address(&self) -> Option<Address> {
        self.0.upgrade().map(|s| s.borrow().address.clone())
    }

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

    pub(crate) fn proc_state<T: Any>(&self) -> Option<Rc<RefCell<T>>> {
        self.state().borrow().get_as()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn alive(&self) -> bool {
        self.0.strong_count() > 0
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
    let timer = timer::sleep(duration);
    timer.await
}

////////////////////////////////////////////////////////////////////////////////

pub fn spawn<F>(task: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
{
    Context::current().spawn(task)
}

////////////////////////////////////////////////////////////////////////////////

pub fn time() -> Time {
    Context::current().event_manager.time()
}

////////////////////////////////////////////////////////////////////////////////

pub fn log(content: impl Into<String>) {
    let context = Context::current();
    let proc = context.proc;
    context.event_manager.add_log(proc, content.into());
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{HashType, Process};

    use super::{Address, ProcessState};

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn string_to_addr() {
        let a: Address = "node:proc".into();
        assert_eq!(a.node, "node");
        assert_eq!(a.process, "proc");
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn get_as() {
        struct Proc {
            x: i32,
        }
        impl Process for Proc {
            fn on_message(&mut self, _from: Address, _content: String) {
                unreachable!()
            }

            fn on_local_message(&mut self, _content: String) {
                unreachable!()
            }

            fn hash(&self) -> HashType {
                0
            }
        }

        let proc_state = ProcessState::new(
            Rc::new(RefCell::new(Proc { x: 1 })),
            Address::new("n1", "p1"),
        );

        let proc = proc_state.get_as::<Proc>().unwrap();
        assert_eq!(proc.borrow().x, 1);

        proc.borrow_mut().x = 2;

        let proc = proc_state.get_as::<Proc>().unwrap();
        assert_eq!(proc.borrow().x, 2);
    }
}
