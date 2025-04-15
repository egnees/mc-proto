use std::hash::Hash;

use crate::simulation::proc::{Address, ProcessHandle};

use super::time::TimeSegment;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct UdpMessageInfo {
    pub udp_msg_id: usize,
    pub from: ProcessHandle,
    pub to: ProcessHandle,
    pub content: String,
    pub can_be_dropped: bool,
}

impl Hash for UdpMessageInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.address().hash(state);
        self.to.address().hash(state);
        self.content.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TimerInfo {
    pub timer_id: usize,
    pub with_sleep: bool,
    pub proc: ProcessHandle,
}

impl Hash for TimerInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.proc.address().hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Hash)]
pub enum EventInfo {
    UdpMessageInfo(UdpMessageInfo),
    TimerInfo(TimerInfo),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Event {
    pub id: usize,
    pub time: TimeSegment,
    pub info: EventInfo,
}

impl Event {
    pub fn new(id: usize, time: TimeSegment, info: EventInfo) -> Self {
        Self { id, time, info }
    }
}

impl Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.time.hash(state);
        self.info.hash(state);
    }
}
