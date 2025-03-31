use std::fmt::{Debug, Display};

use super::control::ApplyFunctor;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct UdpMessage {
    pub udp_msg_id: usize,
    pub drop: bool,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Timer {
    pub timer_id: usize,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum Step {
    SelectUdp(usize, UdpMessage),
    SelectTimer(usize, Timer),
    Apply(Box<dyn ApplyFunctor>),
}

impl Debug for Step {
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

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::SelectUdp(i, udp_message) => {
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
            Step::SelectTimer(i, timer) => {
                write!(f, "Select {}: Timer {} fired", i, timer.timer_id)
            }
            Step::Apply(_) => write!(f, "Apply"),
        }
    }
}
