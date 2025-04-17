use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::Address;

use super::{
    error::Error,
    proc::{Process, ProcessHandle, ProcessState},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Node {
    proc: BTreeMap<String, Rc<RefCell<ProcessState>>>,
    pub(crate) name: String,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            proc: Default::default(),
            name: name.into(),
        }
    }

    pub fn add_proc_by_ref(
        &mut self,
        name: impl Into<String>,
        proc: Rc<RefCell<dyn Process>>,
    ) -> Result<ProcessHandle, Error> {
        let name = name.into();
        if self.proc.contains_key(&name) {
            return Err(Error::AlreadyExists);
        }
        let proc = ProcessState::new(proc, Address::new(self.name.clone(), name.clone()));
        let proc = Rc::new(RefCell::new(proc));
        let handle = ProcessHandle::new(&proc);
        self.proc.insert(name, proc);
        Ok(handle)
    }

    pub fn add_proc(
        &mut self,
        name: impl Into<String>,
        proc: impl Process + 'static,
    ) -> Result<ProcessHandle, Error> {
        self.add_proc_by_ref(name, Rc::new(RefCell::new(proc)))
    }

    pub(crate) fn proc(&self, name: &str) -> Option<ProcessHandle> {
        self.proc.get(name).map(ProcessHandle::new)
    }
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.proc
            .iter()
            .for_each(|(_, proc)| proc.borrow().hash(state));
    }
}
