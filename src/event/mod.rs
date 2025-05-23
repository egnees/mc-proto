use std::{
    fmt::{Debug, Display},
    hash::Hash,
    time::Duration,
};

use info::EventInfo;

use crate::util::trigger::Trigger;

////////////////////////////////////////////////////////////////////////////////

pub mod driver;
pub mod info;
pub mod manager;
pub mod outcome;
pub mod stat;

////////////////////////////////////////////////////////////////////////////////

pub struct Event {
    pub id: usize,
    pub time: Duration,
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

impl Clone for Event {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            time: self.time,
            info: self.info.clone(),
            on_happen: None,
        }
    }
}

impl Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.info.hash(state);
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, ", self.id)?;
        write!(f, "time: {:?}, ", self.time)?;
        write!(f, "info: [{}]", self.info)
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
