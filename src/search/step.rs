use std::{
    fmt::{Debug, Display},
    panic::AssertUnwindSafe,
};

use crate::{
    event::{
        outcome::{EventOutcome, EventOutcomeKind},
        time::TimeSegment,
    },
    SearchError,
};

use super::{control::ApplyFunctor, error::ProcessPanic, state::SearchState};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct UdpMessage {
    pub event_id: usize,
    pub udp_msg_id: usize,
    pub drop: bool,
    pub time: TimeSegment,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Timer {
    pub event_id: usize,
    pub timer_id: usize,
    pub time: TimeSegment,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct TcpPacket {
    pub event_id: usize,
    pub tcp_msg_id: usize,
    pub time: TimeSegment,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum StateTraceStep {
    SelectUdp(usize, UdpMessage),
    SelectTimer(usize, Timer),
    SelectTcp(usize, TcpPacket),
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
    ) -> Result<(), SearchError> {
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
            SearchError::ProcessPanic(p)
        })
    }

    pub fn apply(&self, state: &mut SearchState) -> Result<(), SearchError> {
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
            StateTraceStep::SelectTcp(i, tcp) => {
                let outcome = EventOutcome {
                    event_id: tcp.event_id,
                    kind: EventOutcomeKind::TcpPacketDelivered(),
                    time: tcp.time,
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
            Self::SelectTcp(arg0, arg1) => {
                f.debug_tuple("SelectTcp").field(arg0).field(arg1).finish()
            }
            Self::Apply(_) => f.debug_tuple("Apply").finish(),
            Self::CrashNode(arg0) => f.debug_tuple("CrashNode").field(arg0).finish(),
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
            StateTraceStep::SelectTcp(i, tcp_packet) => {
                write!(
                    f,
                    "Select {}: Tcp packet {} delivered",
                    i, tcp_packet.event_id
                )
            }
            StateTraceStep::CrashNode(node) => {
                write!(f, "Crash node {}", node)
            }
        }
    }
}
