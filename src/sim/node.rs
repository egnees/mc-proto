use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
    time::Duration,
};

use crate::{
    fs::{manager::FsManager, registry::FsEventRegistry},
    Address, Process,
};

use super::{
    error::Error,
    proc::{ProcessHandle, ProcessState},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Node {
    pub(crate) proc: BTreeMap<String, Rc<RefCell<ProcessState>>>,
    pub(crate) fs: Option<FsManager>,
    pub(crate) name: String,
    pub(crate) shutdown: bool,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            proc: Default::default(),
            name: name.into(),
            fs: None,
            shutdown: false,
        }
    }

    pub(crate) fn setup_fs(
        &mut self,
        reg: Rc<RefCell<dyn FsEventRegistry>>,
        min_delay: Duration,
        max_delay: Duration,
        capacity: usize,
    ) -> Result<(), Error> {
        if self.fs.is_some() {
            Err(Error::FsAlreadySetup)
        } else {
            let fs = FsManager::new(reg, self.name.clone(), min_delay, max_delay, capacity);
            let _ = self.fs.insert(fs);
            Ok(())
        }
    }

    pub(crate) fn crash_fs(&mut self) {
        let _ = self.fs.take();
    }

    pub(crate) fn shutdown_fs(&mut self) -> Result<(), Error> {
        self.fs
            .as_ref()
            .ok_or(Error::FsNotAvailable)?
            .handle()
            .shutdown();
        Ok(())
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
        self.proc.iter().for_each(|(name, proc)| {
            name.hash(state);
            proc.borrow().hash(state);
        });

        if let Some(fs) = self.fs.as_ref() {
            fs.hash(state);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct NodeRoleRegister {
    roles: HashMap<String, String>,
}

impl NodeRoleRegister {
    pub fn add(&mut self, node: impl Into<String>, role: impl Into<String>) {
        let prev = self.roles.insert(node.into(), role.into());
        assert!(prev.is_none());
    }

    pub fn remove(&mut self, node: impl Into<String>) -> Option<String> {
        self.roles.remove(node.into().as_str())
        // assert!(prev.is_some());
    }

    pub fn role(&self, node: &str) -> Option<&str> {
        self.roles.get(node).map(|s| s.as_str())
    }
}
