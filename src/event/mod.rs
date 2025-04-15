use std::hash::Hash;

use info::EventInfo;
use time::TimeSegment;

////////////////////////////////////////////////////////////////////////////////

pub mod driver;
pub mod info;
pub mod manager;
pub mod outcome;
pub mod stat;
pub mod time;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Event {
    pub id: usize,
    pub time: TimeSegment,
    pub info: EventInfo,
}

impl Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.time.hash(state);
        self.info.hash(state);
    }
}
