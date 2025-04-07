use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use super::{error::Error, proc::Process};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Node {
    proc: BTreeMap<String, Rc<RefCell<dyn Process>>>,
    local_msg: HashMap<String, Vec<String>>,
}

impl Node {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_proc_by_ref(
        &mut self,
        name: impl Into<String>,
        proc: Rc<RefCell<dyn Process>>,
    ) -> Result<(), Error> {
        let name = name.into();
        if self.proc.contains_key(&name) {
            return Err(Error::AlreadyExists);
        }
        self.proc.insert(name.clone(), proc);
        self.local_msg.insert(name, Default::default());
        Ok(())
    }

    pub fn add_proc(
        &mut self,
        name: impl Into<String>,
        proc: impl Process + 'static,
    ) -> Result<(), Error> {
        self.add_proc_by_ref(name, Rc::new(RefCell::new(proc)))
    }

    pub fn proc(&self, name: &str) -> Option<Rc<RefCell<dyn Process>>> {
        self.proc.get(name).cloned()
    }

    pub fn get_locals(&mut self, name: &str) -> Option<&mut Vec<String>> {
        self.local_msg.get_mut(name)
    }

    pub fn read_locals(&self, name: &str) -> Option<&Vec<String>> {
        self.local_msg.get(name)
    }
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.proc
            .iter()
            .for_each(|(_, proc)| proc.borrow().hash().hash(state));
    }
}
