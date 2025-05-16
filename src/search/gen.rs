use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
    time::Duration,
    usize,
};

use crate::{
    event::{
        driver::EventDriver,
        info::{EventInfo, RpcEventKind, RpcMessageKind, TcpEventKind},
        Event,
    },
    tracker::{EventTracker, MooreEventTracker},
    SystemHandle,
};

use super::{
    config::SearchConfig,
    fs::FsEventKind,
    rpc::{ReadyRpcRequestsFilter, RpcMessageInfo},
    step::{FsEvent, RpcEvent, RpcMessage, StateTraceStep, TcpEvent, TcpPacket, Timer, UdpMessage},
    tcp::{ReadyTcpPacketFilter, TcpPacketKind},
};

////////////////////////////////////////////////////////////////////////////////

enum EventKind {
    UdpMessage(usize),
    Timer(usize),
    TcpPacket(TcpPacketKind),
    TcpEvent(TcpEventKind),
    RpcMessage(RpcMessageInfo),
    RpcEvent(RpcEventKind),
    FsEvent(FsEventKind),
}

////////////////////////////////////////////////////////////////////////////////

pub struct Generator {
    tracker: Option<MooreEventTracker<i64>>,
    event_info: HashMap<usize, EventKind>,
    last_selected: Option<usize>,
}

impl Generator {
    fn last_event_vertex(&self) -> usize {
        self.last_selected.map(|e| e + 1).unwrap_or(0)
    }
}

impl EventDriver for Generator {
    fn hash_pending(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.tracker.as_ref().unwrap().hash_pending(&mut hasher);
        hasher.finish()
    }

    fn register_event(&mut self, event: &Event, min_delay: Duration, max_delay: Duration) {
        let prev = self.last_event_vertex();
        self.tracker.as_mut().unwrap().add_event(
            prev,
            min_delay.as_millis() as i64,
            max_delay.as_millis() as i64,
        );
        let kind =
            match &event.info {
                EventInfo::UdpMessage(msg) => EventKind::UdpMessage(msg.udp_msg_id),
                EventInfo::Timer(timer) => EventKind::Timer(timer.timer_id),
                EventInfo::TcpMessage(msg) => EventKind::TcpPacket(TcpPacketKind {
                    tcp_packet_id: msg.tcp_msg_id,
                    stream: msg.packet.tcp_stream_id,
                    dir: msg.from.address() < msg.to.address(),
                }),
                EventInfo::TcpEvent(e) => EventKind::TcpEvent(e.kind.clone()),
                EventInfo::FsEvent(e) => EventKind::FsEvent(FsEventKind {
                    kind: e.kind.clone(),
                    outcome: e.outcome.clone(),
                }),
                EventInfo::RpcMessage(msg) => match &msg.kind {
                    RpcMessageKind::Request { id, .. } => EventKind::RpcMessage(
                        RpcMessageInfo::new(*id, msg.from.address(), msg.to.address()),
                    ),
                    RpcMessageKind::Response { id, .. } => EventKind::RpcMessage(
                        RpcMessageInfo::new(*id, msg.from.address(), msg.to.address()),
                    ),
                },
                EventInfo::RpcEvent(e) => EventKind::RpcEvent(e.kind.clone()),
            };
        let prev_value = self.event_info.insert(event.id, kind);
        assert!(prev_value.is_none());
    }

    fn cancel_event(&mut self, event: &Event) {
        self.tracker.as_mut().unwrap().cancel_event(event.id + 1);
        let removed = self.event_info.remove(&event.id);
        assert!(removed.is_some());
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            tracker: Some(Default::default()),
            event_info: Default::default(),
            last_selected: Default::default(),
        }
    }
}

impl Generator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_ready_event(&mut self, id: usize) {
        let tracker = self.tracker.take().unwrap();
        self.tracker = Some(tracker.event_happen(id + 1).unwrap());
        self.last_selected = Some(id);
    }

    pub fn steps(&self, system: SystemHandle, cfg: &SearchConfig) -> Vec<StateTraceStep> {
        let mut res = Vec::new();
        let mut tcp_filter = ReadyTcpPacketFilter::new();
        let mut rpc_filter = ReadyRpcRequestsFilter::new();
        for (e, _) in self.tracker.as_ref().unwrap().next_events() {
            let time = self.tracker.as_ref().unwrap().event_time(e);
            assert!(time >= 0);
            let time = Duration::from_millis(time as u64);
            let event_id = e - 1;
            let kind = self.event_info.get(&event_id).unwrap();
            match kind {
                EventKind::UdpMessage(udp_msg_id) => {
                    let udp_msg_id = *udp_msg_id;
                    let udp_no_drop = UdpMessage {
                        event_id,
                        udp_msg_id,
                        drop: false,
                        time,
                    };
                    let no_drop_step = StateTraceStep::SelectUdp(event_id, udp_no_drop);
                    res.push(no_drop_step);

                    // inject msg drop
                    if system.stat().udp_msg_dropped < cfg.max_msg_drops.unwrap_or(usize::MAX) {
                        let udp_drop = UdpMessage {
                            event_id,
                            udp_msg_id,
                            time,
                            drop: true,
                        };
                        let drop_step = StateTraceStep::SelectUdp(event_id, udp_drop);
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
                    let step = StateTraceStep::SelectTimer(event_id, timer);
                    res.push(step);
                }
                EventKind::TcpPacket(tcp) => {
                    let packet = TcpPacket {
                        event_id,
                        time,
                        tcp_msg_id: tcp.tcp_packet_id,
                    };
                    let step = StateTraceStep::SelectTcpPacket(event_id, packet);
                    tcp_filter.add(tcp, step);
                }
                EventKind::TcpEvent(kind) => {
                    let step = StateTraceStep::SelectTcpEvent(
                        event_id,
                        TcpEvent {
                            event_id,
                            time,
                            kind: kind.clone(),
                        },
                    );
                    res.push(step);
                }
                EventKind::RpcMessage(rpc) => {
                    let rpc_msg = RpcMessage {
                        event_id,
                        time,
                        rpc_request_id: rpc.id,
                    };
                    let step = StateTraceStep::SelectRpcMessage(event_id, rpc_msg);
                    rpc_filter.add(rpc, step);
                }
                EventKind::RpcEvent(kind) => {
                    let step = StateTraceStep::SelectRpcEvent(
                        event_id,
                        RpcEvent {
                            event_id,
                            time,
                            kind: kind.clone(),
                        },
                    );
                    res.push(step);
                }
                EventKind::FsEvent(kind) => {
                    let step = StateTraceStep::SelectFsEvent(
                        event_id,
                        FsEvent {
                            event_id,
                            time,
                            outcome: kind.outcome.clone(),
                        },
                    );
                    res.push(step);
                }
            }
        }

        // add tcp packets
        tcp_filter
            .ready_packets()
            .map(|(_, s)| s)
            .cloned()
            .for_each(|s| res.push(s));

        rpc_filter
            .ready_packets()
            .map(|(_, s)| s)
            .cloned()
            .for_each(|s| res.push(s));

        if system.stat().nodes_crashed < cfg.max_node_faults.unwrap_or(usize::MAX) {
            for i in 0..system.nodes_count() {
                res.push(StateTraceStep::CrashNode(i));
            }
        }

        if system.stat().nodes_shutdown < cfg.max_node_shutdown.unwrap_or(usize::MAX) {
            for i in 0..system.nodes_count() {
                if system.node_available_index(i) {
                    res.push(StateTraceStep::ShutdownNode(i));
                }
            }
        }

        res
    }
}
