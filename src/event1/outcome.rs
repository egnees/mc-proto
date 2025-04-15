pub enum EventOutcomeKind {
    UdpMessageDropped(),
    UdpMessageDelivered(),
    TimerFired(),
}

////////////////////////////////////////////////////////////////////////////////

pub struct EventOutcome {
    pub event_id: usize,
    pub kind: EventOutcomeKind,
}
