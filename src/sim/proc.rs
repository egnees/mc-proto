use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::Hash,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{Address, Process};

use super::context::Context;

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

pub fn time() -> Duration {
    Context::current().event_manager.time()
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
