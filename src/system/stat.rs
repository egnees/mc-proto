#[derive(Clone, Debug, Default)]
pub struct Stat {
    pub udp_msg_delivered: usize,
    pub udp_msg_dropped: usize,
}
