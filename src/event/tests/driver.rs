use std::time::Duration;

use crate::event::{driver::EventDriver, Event};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct TestEventDriver {
    pub events: Vec<Event>,
}

impl EventDriver for TestEventDriver {
    fn register_event(&mut self, event: &Event, _min_delay: Duration, _max_delay: Duration) {
        self.events.push(event.clone());
    }

    fn cancel_event(&mut self, event: &Event) {
        let index = self
            .events
            .iter()
            .enumerate()
            .find(|(_, e)| e.id == event.id)
            .map(|(i, _)| i)
            .unwrap();
        self.events.remove(index);
    }

    fn hash_pending(&self) -> u64 {
        0
    }
}

impl TestEventDriver {
    pub fn take(&mut self, idx: usize) -> Event {
        self.events.remove(idx)
    }
}
