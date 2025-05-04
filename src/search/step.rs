use std::{
    fmt::{Debug, Display},
    panic::AssertUnwindSafe,
};

use crate::{
    event::{
        info::TcpEventKind,
        outcome::{EventOutcome, EventOutcomeKind},
        time::Time,
    },
    fs::event::FsEventOutcome,
    SearchErrorKind,
};

use super::{control::ApplyFunctor, error::ProcessPanic, state::SearchState};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct UdpMessage {
    pub event_id: usize,
    pub udp_msg_id: usize,
    pub drop: bool,
    pub time: Time,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Timer {
    pub event_id: usize,
    pub timer_id: usize,
    pub time: Time,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct TcpPacket {
    pub event_id: usize,
    pub tcp_msg_id: usize,
    pub time: Time,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct TcpEvent {
    pub event_id: usize,
    pub time: Time,
    pub kind: TcpEventKind,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct FsEvent {
    pub event_id: usize,
    pub time: Time,
    pub outcome: FsEventOutcome,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum StateTraceStep {
    SelectUdp(usize, UdpMessage),
    SelectTimer(usize, Timer),
    SelectTcpPacket(usize, TcpPacket),
    SelectTcpEvent(usize, TcpEvent),
    SelectFsEvent(usize, FsEvent),
    CrashNode(usize), // id of node
    Apply(Box<dyn ApplyFunctor>),
}

////////////////////////////////////////////////////////////////////////////////

impl StateTraceStep {
    fn apply_event_outcome(
        &self,
        state: &mut SearchState,
        i: usize,
        outcome: EventOutcome,
    ) -> Result<(), SearchErrorKind> {
        state.gen.borrow_mut().select_ready_event(i);
        let handle = state.system.handle();
        std::panic::catch_unwind(AssertUnwindSafe(move || {
            handle.handle_event_outcome(outcome);
        }))
        .map_err(|_| {
            let p = ProcessPanic {
                trace: None,
                log: state.system.handle().log(),
            };
            SearchErrorKind::ProcessPanic(p)
        })
    }

    pub fn apply(&self, state: &mut SearchState) -> Result<(), SearchErrorKind> {
        match self {
            StateTraceStep::SelectUdp(i, msg) => {
                let kind = if msg.drop {
                    EventOutcomeKind::UdpMessageDropped()
                } else {
                    EventOutcomeKind::UdpMessageDelivered()
                };
                let outcome = EventOutcome {
                    event_id: msg.event_id,
                    kind,
                    time: msg.time,
                };
                self.apply_event_outcome(state, *i, outcome)
            }
            StateTraceStep::SelectTimer(i, timer) => {
                let outcome = EventOutcome {
                    event_id: timer.event_id,
                    kind: EventOutcomeKind::TimerFired(),
                    time: timer.time,
                };
                self.apply_event_outcome(state, *i, outcome)
            }
            StateTraceStep::SelectTcpPacket(i, tcp) => {
                let outcome = EventOutcome {
                    event_id: tcp.event_id,
                    kind: EventOutcomeKind::TcpPacketDelivered(),
                    time: tcp.time,
                };
                self.apply_event_outcome(state, *i, outcome)
            }
            StateTraceStep::SelectTcpEvent(i, e) => {
                let tcp_result = e.kind.tcp_result();
                let outcome = EventOutcome {
                    event_id: e.event_id,
                    kind: EventOutcomeKind::TcpEventHappen(tcp_result),
                    time: e.time,
                };
                self.apply_event_outcome(state, *i, outcome)
            }
            StateTraceStep::Apply(f) => {
                f.apply(state.system.handle());
                Ok(())
            }
            StateTraceStep::CrashNode(node) => {
                state.system.handle().crash_node_index(*node);
                Ok(())
            }
            StateTraceStep::SelectFsEvent(i, e) => {
                let outcome = EventOutcome {
                    event_id: e.event_id,
                    kind: EventOutcomeKind::FsEventHappen(e.outcome.clone()),
                    time: e.time,
                };
                self.apply_event_outcome(state, *i, outcome)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Debug for StateTraceStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelectUdp(arg0, arg1) => {
                f.debug_tuple("SelectUdp").field(arg0).field(arg1).finish()
            }
            Self::SelectTimer(arg0, arg1) => f
                .debug_tuple("SelectTimer")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::SelectTcpPacket(arg0, arg1) => {
                f.debug_tuple("SelectTcp").field(arg0).field(arg1).finish()
            }
            Self::Apply(_) => f.debug_tuple("Apply").finish(),
            Self::CrashNode(arg0) => f.debug_tuple("CrashNode").field(arg0).finish(),
            Self::SelectTcpEvent(arg0, arg1) => f
                .debug_tuple("SelectTcpEvent")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::SelectFsEvent(arg0, arg1) => f
                .debug_tuple("SelectFsEvent")
                .field(arg0)
                .field(arg1)
                .finish(),
        }
    }
}

impl Display for StateTraceStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateTraceStep::SelectUdp(i, udp_message) => {
                if udp_message.drop {
                    write!(
                        f,
                        "Select {}: UDP message {} dropped",
                        i, udp_message.udp_msg_id
                    )
                } else {
                    write!(
                        f,
                        "Select {}: UDP message {} delivered",
                        i, udp_message.udp_msg_id
                    )
                }
            }
            StateTraceStep::SelectTimer(i, timer) => {
                write!(f, "Select {}: Timer {} fired", i, timer.timer_id)
            }
            StateTraceStep::Apply(_) => write!(f, "Apply"),
            StateTraceStep::SelectTcpPacket(i, tcp_packet) => {
                write!(
                    f,
                    "Select {}: Tcp packet {} delivered",
                    i, tcp_packet.event_id
                )
            }
            StateTraceStep::CrashNode(node) => {
                write!(f, "Crash node {}", node)
            }
            StateTraceStep::SelectTcpEvent(i, _) => {
                write!(f, "Select {}: Tcp event", *i)
            }
            StateTraceStep::SelectFsEvent(i, _) => {
                write!(f, "Select {}: Fs event", *i)
            }
        }
    }
}
