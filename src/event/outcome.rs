use std::time::Duration;

use crate::{fs::event::FsEventOutcome, rpc::RpcResult, TcpError};

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
