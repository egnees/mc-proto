use std::time::Duration;

use crate::{model::fs::event::FsEventOutcome, model::tcp::TcpError, RpcResult};

////////////////////////////////////////////////////////////////////////////////

pub enum EventOutcomeKind {
    UdpMessageDropped(),
    UdpMessageDelivered(),
    TimerFired(),
    TcpPacketDelivered(),
    TcpEventHappen(Result<(), TcpError>),
    RpcMessageDelivered,
    RpcEventHappen(RpcResult<()>),
    FsEventHappen(FsEventOutcome),
}

////////////////////////////////////////////////////////////////////////////////

pub struct EventOutcome {
    pub event_id: usize,
    pub kind: EventOutcomeKind,
    pub time: Duration,
}
