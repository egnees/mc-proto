use super::control::ApplyFunctor;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct UdpMessage {
    pub udp_msg_id: usize,
    pub drop: bool,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
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
