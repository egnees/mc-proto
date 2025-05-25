use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    model::fs::{
        event::{FsEvent, FsEventOutcome},
        registry::FsEventRegistry,
    },
    util::trigger::Trigger,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct DelayedRegister {
    pub events: Vec<(Trigger, FsEventOutcome)>,
}

impl FsEventRegistry for DelayedRegister {
    fn register_instant_event(&mut self, _event: &FsEvent) {
        // do nothing
    }

    fn register_event_initiated(&mut self, _event: &FsEvent) {
        // do nothing
    }

    fn register_event_pipelined(
        &mut self,
        trigger: Trigger,
        event: &FsEvent,
        _min_delay: Duration,
        _max_delay: Duration,
    ) {
        self.events.push((trigger, event.outcome.clone()));
    }

    fn register_event_happen(&mut self, _event: &FsEvent) {
        // do nothing
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_delayed_register() -> Rc<RefCell<DelayedRegister>> {
    Rc::new(RefCell::new(DelayedRegister::default()))
}
