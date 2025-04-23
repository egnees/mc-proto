use std::{collections::HashMap, time::Duration};

use crate::{
    event::{driver::EventDriver, info::EventInfo, time::TimeSegment, Event},
    SystemHandle,
};

use super::{
    config::SearchConfig,
    step::{StateTraceStep, TcpPacket, Timer, UdpMessage},
    tcp::{ReadyTcpPacketFilter, TcpPacketKind},
    tracker::Tracker,
};

////////////////////////////////////////////////////////////////////////////////

enum EventKind {
    UdpMessage(usize),
    Timer(usize),
    TcpPacket(TcpPacketKind),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Generator {
    tracker: Tracker<Duration>,
    event_info: HashMap<usize, EventKind>,
}

impl EventDriver for Generator {
    fn register_event(&mut self, event: &Event) {
        let kind = match &event.info {
            EventInfo::UdpMessage(msg) => EventKind::UdpMessage(msg.udp_msg_id),
            EventInfo::Timer(timer) => EventKind::Timer(timer.timer_id),
            EventInfo::TcpMessage(msg) => EventKind::TcpPacket(TcpPacketKind {
                tcp_packet_id: msg.tcp_msg_id,
                stream: msg.packet.tcp_stream_id,
                dir: msg.from.address() < msg.to.address(),
            }),
        };
        let prev_value = self.event_info.insert(event.id, kind);
        assert!(prev_value.is_none());
        self.tracker.add(event.time.from, event.time.to, event.id);
    }

    fn cancel_event(&mut self, event: &Event) {
        let removed = self.tracker.remove_by_event_id(event.id);
        assert!(removed.is_some());
        let removed = self.event_info.remove(&event.id);
        assert!(removed.is_some());
    }
}

impl Generator {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn select_ready_event(&mut self, i: usize) {
        self.tracker.remove_ready(i);
    }

    pub fn steps(&self, system: SystemHandle, cfg: &SearchConfig) -> Vec<StateTraceStep> {
        let pending = self.tracker.ready_count();
        let mut res = Vec::new();
        let mut tcp_filter = ReadyTcpPacketFilter::new();
        for i in 0..pending {
            let e = self.tracker.get_ready(i).unwrap();
            let time = TimeSegment::new(e.from, e.to);
            let event_id = e.event_id;
            let kind = self.event_info.get(&e.event_id).unwrap();
            match kind {
                EventKind::UdpMessage(udp_msg_id) => {
                    let udp_msg_id = *udp_msg_id;
                    let udp_no_drop = UdpMessage {
                        event_id,
                        udp_msg_id,
                        drop: false,
                        time,
                    };
                    let no_drop_step = StateTraceStep::SelectUdp(i, udp_no_drop);
                    res.push(no_drop_step);

                    // inject msg drop
                    if system.stat().udp_msg_dropped < cfg.max_msg_drops.unwrap_or(usize::MAX) {
                        let udp_drop = UdpMessage {
                            event_id,
                            udp_msg_id,
                            time,
                            drop: true,
                        };
                        let drop_step = StateTraceStep::SelectUdp(i, udp_drop);
                        res.push(drop_step);
                    }
                }
                EventKind::Timer(timer_id) => {
                    let timer_id = *timer_id;
                    let timer = Timer {
                        event_id,
                        time,
                        timer_id,
                    };
                    let step = StateTraceStep::SelectTimer(i, timer);
                    res.push(step);
                }
                EventKind::TcpPacket(tcp) => {
                    let packet = TcpPacket {
                        event_id,
                        time,
                        tcp_msg_id: tcp.tcp_packet_id,
                    };
                    let step = StateTraceStep::SelectTcp(i, packet);
                    tcp_filter.add(tcp, step);
                }
            }
        }

        // add tcp packets
        tcp_filter
            .ready_packets()
            .map(|(_, s)| s)
            .cloned()
            .for_each(|s| res.push(s));

        if system.stat().nodes_crashed < cfg.max_node_faults.unwrap_or(usize::MAX) {
            for i in 0..system.nodes_count() {
                res.push(StateTraceStep::CrashNode(i));
            }
        }
        res
    }
}
