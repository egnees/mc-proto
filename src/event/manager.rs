use std::time::Duration;

use crate::{event::tracker::Tracker, simulation::proc::Address};

use super::{
    info::{Event, EventInfo, TimerInfo, UdpMessageInfo},
    time::TimeSegment,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Manager {
    tracker: Tracker<Duration>,
    events: Vec<Event>,
    udp_msg_cnt: usize,
    timers_cnt: usize,
}

impl Manager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_udp_message(
        &mut self,
        from: Address,
        to: Address,
        content: String,
        time: TimeSegment,
    ) -> &Event {
        let id = self.next_event_id();
        self.tracker.add(time.from, time.to, id);
        let info = UdpMessageInfo {
            udp_msg_id: self.udp_msg_cnt,
            from,
            to,
            content,
        };
        let info = EventInfo::UdpMessageInfo(info);
        let event = Event::new(id, time, info);
        self.udp_msg_cnt += 1;
        self.events.push(event);
        self.events.last().unwrap()
    }

    pub fn register_timer(&mut self, proc: Address, with_sleep: bool, time: TimeSegment) -> &Event {
        let id = self.next_event_id();
        self.tracker.add(time.from, time.to, id);
        let info = TimerInfo {
            timer_id: self.timers_cnt,
            with_sleep,
            proc,
        };
        let info = EventInfo::TimerInfo(info);
        let event = Event::new(id, time, info);
        self.timers_cnt += 1;
        self.events.push(event);
        self.events.last().unwrap()
    }

    pub fn get(&self, id: usize) -> Option<&Event> {
        self.events.get(id)
    }

    pub fn ready_events_cnt(&self) -> usize {
        self.tracker.ready_count()
    }

    pub fn get_ready(&self, i: usize) -> Option<Event> {
        let seg = self.tracker.get_ready(i)?;
        let mut e = self.get(seg.event_id)?.clone();
        e.time.from = seg.from;
        e.time.to = seg.to;
        Some(e)
    }

    pub fn remove_ready(&mut self, i: usize) -> Option<Event> {
        let seg = self.tracker.remove_ready(i)?;
        let mut e = self.get(seg.event_id)?.clone();
        e.time.from = seg.from;
        e.time.to = seg.to;
        Some(e)
    }

    fn next_event_id(&self) -> usize {
        self.events.len()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl std::hash::Hash for Manager {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let pending = self.tracker.ready_count();
        (0..pending)
            .map(|i| self.tracker.get_ready(i).unwrap().event_id)
            .for_each(|i| self.events[i].hash(state));
    }
}
