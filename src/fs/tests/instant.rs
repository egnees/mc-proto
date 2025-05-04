use std::{cell::RefCell, rc::Rc};

use crate::{
    event::time::Time,
    fs::{event::FsEvent, registry::FsEventRegistry},
    util::trigger::Trigger,
    Log,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct InstantRegister {
    pub log: Log,
}

impl FsEventRegistry for InstantRegister {
    fn register_instant_event(&mut self, event: &FsEvent) {
        let e = event.clone().make_log_entry_on_init(Time::default_range());
        self.log.add_entry(e);
    }

    fn register_event_initiated(&mut self, event: &FsEvent) {
        let e = event.clone().make_log_entry_on_init(Time::default_range());
        self.log.add_entry(e);
    }

    fn register_event_pipelined(&mut self, trigger: Trigger, event: &FsEvent) {
        let _ = trigger.invoke(event.outcome.clone());
    }

    fn register_event_happen(&mut self, event: &FsEvent) {
        let e = event
            .clone()
            .make_log_entry_on_complete(Time::default_range());
        self.log.add_entry(e);
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_shared_instant() -> Rc<RefCell<InstantRegister>> {
    Rc::new(RefCell::new(InstantRegister::default()))
}
