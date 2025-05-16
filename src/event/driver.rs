use std::time::Duration;

use super::Event;

////////////////////////////////////////////////////////////////////////////////

pub trait EventDriver {
    fn register_event(&mut self, event: &Event, min_offset: Duration, max_offset: Duration);

    fn cancel_event(&mut self, event: &Event);

    fn hash_pending(&self) -> u64;
}
