#[derive(Clone, Default)]
pub struct EventStat {
    pub udp_msg_dropped: usize,
    pub nodes_crashed: usize,
    pub nodes_shutdown: usize,
}
