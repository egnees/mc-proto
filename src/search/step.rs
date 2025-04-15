use std::fmt::{Debug, Display};

use crate::event::outcome::{EventOutcome, EventOutcomeKind};

use super::{control::ApplyFunctor, state::SearchState};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct UdpMessage {
    pub event_id: usize,
    pub udp_msg_id: usize,
    pub drop: bool,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Timer {
    pub event_id: usize,
    pub timer_id: usize,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum StateTraceStep {
    SelectUdp(usize, UdpMessage),
    SelectTimer(usize, Timer),
    Apply(Box<dyn ApplyFunctor>),
}

////////////////////////////////////////////////////////////////////////////////

impl StateTraceStep {
    pub fn apply(&self, state: &mut SearchState) {
        match self {
            StateTraceStep::SelectUdp(i, msg) => {
                state.gen.borrow_mut().select_ready_event(*i);
                let kind = if msg.drop {
                    EventOutcomeKind::UdpMessageDropped()
                } else {
                    EventOutcomeKind::UdpMessageDelivered()
                };
                let outcome = EventOutcome {
                    event_id: msg.event_id,
                    kind,
                };
                state.system.handle().handle_event_outcome(outcome);
            }
            StateTraceStep::SelectTimer(i, timer) => {
                state.gen.borrow_mut().select_ready_event(*i);
                let kind = EventOutcomeKind::TimerFired();
                let outcome = EventOutcome {
                    event_id: timer.event_id,
                    kind,
                };
                state.system.handle().handle_event_outcome(outcome);
            }
            StateTraceStep::Apply(f) => {
                f.apply(state.system.handle());
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
            Self::Apply(_) => f.debug_tuple("Apply").finish(),
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
        }
    }
}
