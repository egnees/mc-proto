use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    hash::Hash,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{util::trigger::Waiter, Address};

use super::{
    disk::Disk,
    error::FsError,
    event::{FsEvent, FsEventKind, FsEventOutcome},
    file::{File, FileContent},
    registry::FsEventRegistry,
};

////////////////////////////////////////////////////////////////////////////////

struct FsManagerState {
    reg: Rc<RefCell<dyn FsEventRegistry>>,
    disk: Disk,
    files: BTreeMap<String, Rc<RefCell<FileContent>>>,
    node: String,
    available: bool,
}

impl FsManagerState {
    pub fn new(
        reg: Rc<RefCell<dyn FsEventRegistry>>,
        node: String,
        min_disk_delay: Duration,
        max_disk_delay: Duration,
        disk_capacity: usize,
    ) -> Self {
        let disk = Disk::new(
            reg.clone(),
            min_disk_delay,
            max_disk_delay,
            node.clone(),
            disk_capacity,
        );
        Self {
            reg,
            disk,
            files: Default::default(),
            node,
            available: true,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct FsManager(Rc<RefCell<FsManagerState>>);

impl FsManager {
    pub fn new(
        reg: Rc<RefCell<dyn FsEventRegistry>>,
        node: String,
        min_disk_delay: Duration,
        max_disk_delay: Duration,
        disk_capacity: usize,
    ) -> Self {
        let state = FsManagerState::new(reg, node, min_disk_delay, max_disk_delay, disk_capacity);
        Self(Rc::new(RefCell::new(state)))
    }

    pub fn handle(&self) -> FsManagerHandle {
        FsManagerHandle(Rc::downgrade(&self.0))
    }
}

impl Hash for FsManager {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct FsManagerHandle(Weak<RefCell<FsManagerState>>);

impl FsManagerHandle {
    fn state(&self) -> Rc<RefCell<FsManagerState>> {
        self.0.upgrade().unwrap()
    }

    pub fn open_file(
        &self,
        proc: String,
        name: String,
    ) -> Result<Weak<RefCell<FileContent>>, FsError> {
        let state = self.state();
        let state = state.borrow_mut();

        let outcome = {
            let availabe = state.available;
            if !availabe {
                Err(FsError::StorageNotAvailable)
            } else {
                state
                    .files
                    .get(&name)
                    .ok_or(FsError::FileNotFound { file: name.clone() })
                    .map(Rc::downgrade)
            }
        };

        state.reg.borrow_mut().register_event_initiated(&FsEvent {
            initiated_by: Address::new(state.node.clone(), proc),
            kind: FsEventKind::Open { file: name },
            outcome: outcome.clone().map(|_| ()),
        });

        outcome
    }

    pub fn create_file(
        &self,
        proc: String,
        name: String,
    ) -> Result<Weak<RefCell<FileContent>>, FsError> {
        let state = self.state();
        let mut state = state.borrow_mut();
        let outcome = {
            if !state.available {
                Err(FsError::StorageNotAvailable)
            } else {
                let entry = state.files.entry(name.clone());
                if let Entry::Vacant(entry) = entry {
                    let content = Rc::downgrade(entry.insert(Default::default()));
                    Ok(content)
                } else {
                    Err(FsError::FileAlreadyExists { file: name.clone() })
                }
            }
        };

        state.reg.borrow_mut().register_instant_event(&FsEvent {
            initiated_by: Address::new(state.node.clone(), proc),
            kind: FsEventKind::Create { file: name },
            outcome: outcome.clone().map(|_| ()),
        });

        outcome
    }

    pub fn delete_file(&self, proc: String, name: String) -> Result<(), FsError> {
        let state = self.state();
        let mut state = state.borrow_mut();
        let outcome = {
            if !state.available {
                Err(FsError::StorageNotAvailable)
            } else {
                let content = state.files.remove(&name);
                if let Some(content) = content {
                    state.disk.file_deleted(content.borrow().size());
                    Ok(())
                } else {
                    Err(FsError::FileNotFound {
                        file: name.to_string(),
                    })
                }
            }
        };
        state.reg.borrow_mut().register_instant_event(&FsEvent {
            initiated_by: Address::new(state.node.clone(), proc),
            kind: FsEventKind::Delete {
                file: name.to_string(),
            },
            outcome: outcome.clone(),
        });
        outcome
    }

    pub fn register_async_file_event(
        &self,
        file: &File,
        kind: FsEventKind,
    ) -> Result<Waiter, FsError> {
        let available = self.state().borrow().available;
        if available {
            Ok(self
                .state()
                .borrow_mut()
                .disk
                .enqueue_request(file.owner_proc.clone(), kind))
        } else {
            Err(FsError::StorageNotAvailable)
        }
    }

    pub fn register_event_happen(&self, file: &File, kind: FsEventKind, outcome: FsEventOutcome) {
        self.state()
            .borrow_mut()
            .disk
            .on_request_completed(file.owner_proc.clone(), kind, outcome);
    }

    pub fn crash(&self) {
        let state = self.state();
        let mut state = state.borrow_mut();
        state.disk.crash();
        state.files.clear();
        state.available = false;
    }

    pub fn raise(&self) {
        let state = self.state();
        let mut state = state.borrow_mut();
        state.available = true;
    }

    pub fn shutdown(&self) {
        let state = self.state();
        let mut state = state.borrow_mut();
        state.disk.shutdown();
        state.available = false;
    }

    pub fn available(&self) -> bool {
        self.state().borrow().available
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Hash for FsManagerState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (name, content) in self.files.iter() {
            name.hash(state);
            content.borrow().hash(state);
        }
    }
}
