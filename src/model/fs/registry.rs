use std::time::Duration;

use crate::util::trigger::Trigger;

use super::event::FsEvent;

////////////////////////////////////////////////////////////////////////////////

pub trait FsEventRegistry {
    fn register_instant_event(&mut self, event: &FsEvent);
    fn register_event_initiated(&mut self, event: &FsEvent);
    fn register_event_pipelined(
        &mut self,
        trigger: Trigger,
        event: &FsEvent,
        min_delay: Duration,
        max_delay: Duration,
    );
    fn register_event_happen(&mut self, event: &FsEvent);
}
