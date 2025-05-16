use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    event::{
        driver::EventDriver,
        info::EventInfo,
        outcome::{EventOutcome, EventOutcomeKind},
        Event,
    },
    Address,
};

use super::StepConfig;

////////////////////////////////////////////////////////////////////////////////

pub struct Driver {
    info: BTreeMap<usize, (Duration, EventInfo)>,
    queue: BTreeSet<(Duration, usize)>,
    rng: SmallRng,
    last_tcp: BTreeMap<(usize, bool), Duration>,
    last_rpc: BTreeMap<(Address, Address), Duration>,
}

impl EventDriver for Driver {
    fn register_event(&mut self, event: &Event, min_offset: Duration, max_offset: Duration) {
        assert!(min_offset <= max_offset);
        let t = {
            let from = match &event.info {
                EventInfo::RpcMessage(m) => self
                    .last_rpc
                    .get(&(m.from.address(), m.to.address()))
                    .cloned()
                    .unwrap_or(Duration::ZERO)
                    .max(min_offset),
                EventInfo::TcpMessage(m) => self
                    .last_tcp
                    .get(&(m.packet.tcp_stream_id, m.from.address() < m.to.address()))
                    .cloned()
                    .unwrap_or(Duration::ZERO)
                    .max(min_offset),
                _ => min_offset,
            };
            self.rng.random_range(from..max_offset)
        };

        match &event.info {
            EventInfo::RpcMessage(m) => {
                self.last_rpc.insert((m.from.address(), m.to.address()), t);
            }
            EventInfo::TcpMessage(m) => {
                self.last_tcp.insert(
                    (m.packet.tcp_stream_id, m.from.address() < m.to.address()),
                    t,
                );
            }
            _ => {}
        };

        let prev = self.info.insert(event.id, (t, event.info.clone()));
        assert!(prev.is_none());

        let not_exist = self.queue.insert((t, event.id));
        assert!(not_exist);
    }

    fn cancel_event(&mut self, event: &Event) {
        let (t, _) = self.info.remove(&event.id).unwrap();
        let exist = self.queue.remove(&(t, event.id));
        assert!(exist);
    }

    fn hash_pending(&self) -> u64 {
        0
    }
}

impl Driver {
    pub fn new(seed: u64) -> Self {
        Self {
            last_rpc: Default::default(),
            last_tcp: Default::default(),
            info: Default::default(),
            queue: Default::default(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn next_event_outcome(&mut self, cfg: &StepConfig) -> Option<EventOutcome> {
        if let Some((time, event_id)) = self.queue.pop_first() {
            let (t, event_info) = self.info.remove(&event_id).unwrap();
            assert_eq!(t, time);
            let kind = match &event_info {
                EventInfo::UdpMessage(_) => {
                    let dropped = self.rng.random_range(0.0..1.0) < cfg.udp_packet_drop_prob;
                    if dropped {
                        EventOutcomeKind::UdpMessageDropped()
                    } else {
                        EventOutcomeKind::UdpMessageDelivered()
                    }
                }
                EventInfo::TcpMessage(_) => EventOutcomeKind::TcpPacketDelivered(),
                EventInfo::TcpEvent(tcp_event) => {
                    EventOutcomeKind::TcpEventHappen(tcp_event.kind.tcp_result())
                }
                EventInfo::FsEvent(fs_event) => {
                    EventOutcomeKind::FsEventHappen(fs_event.outcome.clone())
                }
                EventInfo::Timer(_) => EventOutcomeKind::TimerFired(),
                EventInfo::RpcMessage(_) => EventOutcomeKind::RpcMessageDelivered,
                EventInfo::RpcEvent(e) => EventOutcomeKind::RpcEventHappen(e.kind.rpc_result()),
            };
            let outcome = EventOutcome {
                event_id,
                kind,
                time,
            };
            Some(outcome)
        } else {
            None
        }
    }
}
