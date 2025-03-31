use std::time::Duration;

use super::proc::Address;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UdpMessageSent {
    pub from: Address,
    pub to: Address,
    pub content: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UdpMessageReceived {
    pub from: Address,
    pub to: Address,
    pub content: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UdpMessageDropped {
    pub from: Address,
    pub to: Address,
    pub content: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessFellAsleep {
    pub proc: Address,
    pub duration: Duration,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessWokeUp {
    pub proc: Address,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessSentLocalMessage {
    pub process: Address,
    pub content: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessReceivedLocalMessage {
    pub process: Address,
    pub content: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum LogEntry {
    UdpMessageSent(UdpMessageSent),
    UdpMessageReceived(UdpMessageReceived),
    UdpMessageDropped(UdpMessageDropped),
    ProcessSentLocalMessage(ProcessSentLocalMessage),
    ProcessReceivedLocalMessage(ProcessReceivedLocalMessage),
    ProcessFellAsleep(ProcessFellAsleep),
    ProcessWokeUp(ProcessWokeUp),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct Log {
    data: Vec<LogEntry>,
}

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, log_entry: LogEntry) {
        self.data.push(log_entry);
    }
}
