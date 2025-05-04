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
        time::Time,
        Event,
    },
    SystemHandle,
};

use super::StepConfig;

////////////////////////////////////////////////////////////////////////////////

pub struct Driver {
    info: BTreeMap<usize, (Duration, EventInfo)>,
    queue: BTreeSet<(Duration, usize)>,
    rng: SmallRng,
}

impl EventDriver for Driver {
    fn register_event(&mut self, event: &Event) {
        let t = match event.time {
            Time::Point(duration) => duration,
            Time::Segment(time_segment) => {
                self.rng.random_range(time_segment.from..time_segment.to)
            }
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

    fn start_time(&self) -> Time {
        Time::default_point()
    }
}

impl Driver {
    pub fn new(seed: u64) -> Self {
        Self {
            info: Default::default(),
            queue: Default::default(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn make_step(&mut self, system: SystemHandle, cfg: &StepConfig) -> bool {
        if let Some((time, event_id)) = self.queue.pop_first() {
            let (t, event_info) = self.info.remove(&event_id).unwrap();
            assert_eq!(t, time);
            let time = Time::new_point(time);
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
            };
            let outcome = EventOutcome {
                event_id,
                kind,
                time,
            };
            system.handle_event_outcome(outcome);
            true
        } else {
            false
        }
    }
}
