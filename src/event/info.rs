use std::hash::Hash;

use crate::system::proc::Address;

use super::time::TimeSegment;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct UdpMessageInfo {
    pub udp_msg_id: usize,
    pub from: Address,
    pub to: Address,
    pub content: String,
}

impl Hash for UdpMessageInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.content.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TimerInfo {
    pub timer_id: usize,
    pub with_sleep: bool,
    pub proc: Address,
}

impl Hash for TimerInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.proc.hash(state);
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
