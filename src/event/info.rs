use std::{hash::Hash, time::Duration};

use crate::simulation::proc::ProcessHandle;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Hash)]
pub enum EventInfo {
    UdpMessage(UdpMessage),
    Timer(Timer),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct UdpMessage {
    pub udp_msg_id: usize,
    pub from: ProcessHandle,
    pub to: ProcessHandle,
    pub content: String,
}

impl Hash for UdpMessage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.address().hash(state);
        self.to.address().hash(state);
        self.content.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Timer {
    pub timer_id: usize,
    pub with_sleep: bool,
    pub proc: ProcessHandle,
    pub duration: Duration,
}

impl Hash for Timer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.duration.hash(state);
    }
}
