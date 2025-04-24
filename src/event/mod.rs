use std::hash::Hash;

use info::EventInfo;
use time::TimeSegment;

use crate::util::trigger::Trigger;

////////////////////////////////////////////////////////////////////////////////

pub mod driver;
pub mod info;
pub mod manager;
pub mod outcome;
pub mod stat;
pub mod time;

////////////////////////////////////////////////////////////////////////////////

pub struct Event {
    pub id: usize,
    pub time: TimeSegment,
    pub info: EventInfo,
    pub on_happen: Option<Trigger>,
}

impl Event {
    pub fn cloned(&mut self) -> Self {
        Self {
            id: self.id,
            time: self.time,
            info: self.info.clone(),
            on_happen: self.on_happen.take(),
        }
    }
}

impl Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.info.hash(state);
    }
}
