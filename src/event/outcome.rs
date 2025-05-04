use crate::{fs::event::FsEventOutcome, TcpError};

use super::time::Time;

////////////////////////////////////////////////////////////////////////////////

pub enum EventOutcomeKind {
    UdpMessageDropped(),
    UdpMessageDelivered(),
    TimerFired(),
    TcpPacketDelivered(),
    TcpEventHappen(Result<(), TcpError>),
    FsEventHappen(FsEventOutcome),
}

////////////////////////////////////////////////////////////////////////////////

pub struct EventOutcome {
    pub event_id: usize,
    pub kind: EventOutcomeKind,
    pub time: Time,
}
