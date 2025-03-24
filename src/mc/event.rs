use std::fmt::Display;

use super::{
    message::{Message, MessageId},
    process::ProcessId,
    time::TimerId,
};

////////////////////////////////////////////////////////////////////////////////

pub enum EventId {
    TimerFired(TimerId),
    MessageReceived(MessageId),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct PendingEvents {
    events: Vec<EventId>,
}

impl PendingEvents {
    pub fn add(&mut self, event: EventId) {
        self.events.push(event);
    }

    pub fn events_count(&self) -> usize {
        self.events.len()
    }

    pub fn take_event(&mut self, at: usize) -> EventId {
        self.events.remove(at)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TimerSetEvent {
    pub timer_id: TimerId,
    pub proc: ProcessId,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TimerFiredEvent {
    pub timer_id: TimerId,
    pub proc: ProcessId,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MessageSentEvent {
    pub message_id: MessageId,
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub content: Message,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MessageReceivedEvent {
    pub message_id: MessageId,
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub content: Message,
}

////////////////////////////////////////////////////////////////////////////////

/// Process sent local message
#[derive(Debug, Clone)]
pub struct LocalMessageSentEvent {
    pub message_id: MessageId,
    pub process: ProcessId,
    pub content: Message,
}

////////////////////////////////////////////////////////////////////////////////

/// Process received local message
#[derive(Debug, Clone)]
pub struct LocalMessageReceivedEvent {
    pub message_id: MessageId,
    pub process: ProcessId,
    pub content: Message,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MessageDroppedEvent {
    pub message_id: MessageId,
    pub sender: ProcessId,
    pub receiver: ProcessId,
    pub content: Message,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum Event {
    TimerSet(TimerSetEvent),
    TimerFired(TimerFiredEvent),
    MessageSent(MessageSentEvent),
    MessageDropped(MessageDroppedEvent),
    MessageReceived(MessageReceivedEvent),
    LocalMessageSent(LocalMessageSentEvent),
    LocalMessageReceived(LocalMessageReceivedEvent),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "")
        match self {
            Event::TimerSet(event) => write!(f, "Set T#{}, P#{}", event.timer_id, event.proc),
            Event::TimerFired(event) => {
                write!(f, "T#{} fired on P#{}", event.timer_id, event.proc)
            }
            Event::MessageSent(event) => write!(
                f,
                "Sent M#{}, from P#{} to P#{}: \"{}\"",
                event.message_id, event.sender, event.receiver, event.content
            ),
            Event::MessageDropped(event) => write!(
                f,
                "Dropped M#{}, from P#{} to P#{}: \"{}\"",
                event.message_id, event.sender, event.receiver, event.content
            ),
            Event::MessageReceived(event) => write!(
                f,
                "P#{} received M#{} from P#{}: \"{}\"",
                event.receiver, event.message_id, event.sender, event.content
            ),
            Event::LocalMessageSent(event) => write!(
                f,
                "P#{} sent LM#{}: \"{}\"",
                event.process, event.message_id, event.content
            ),
            Event::LocalMessageReceived(event) => write!(
                f,
                "P#{} received LM#{}: \"{}\"",
                event.process, event.message_id, event.content
            ),
        }
    }
}
