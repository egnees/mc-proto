use std::time::Duration;

use crate::{system::proc::Address, track::tracker::Tracker};

use super::info::{Event, EventInfo, TimerInfo, UdpMessageInfo};

////////////////////////////////////////////////////////////////////////////////

pub struct Manager {
    tracker: Box<dyn Tracker<Duration>>,
    events: Vec<Event>,
    udp_msg_cnt: usize,
    timers_cnt: usize,
}

impl Manager {
    pub fn new(tracker: impl Tracker<Duration> + 'static) -> Self {
        Self {
            tracker: Box::new(tracker),
            events: Vec::new(),
            udp_msg_cnt: 0,
            timers_cnt: 0,
        }
    }

    pub fn register_udp_message(
        &mut self,
        from: Address,
        to: Address,
        content: String,
        time_from: Duration,
        time_to: Duration,
    ) -> &Event {
        let id = self.next_event_id();
        self.tracker.add(time_from, time_to, id);
        let info = UdpMessageInfo {
            udp_msg_id: self.udp_msg_cnt,
            from,
            to,
            content,
        };
        let info = EventInfo::UdpMessageInfo(info);
        let event = Event {
            id,
            time_from,
            time_to,
            info,
        };
        self.udp_msg_cnt += 1;
        self.events.push(event);
        self.events.last().unwrap()
    }

    pub fn register_timer(
        &mut self,
        proc: Address,
        with_sleep: bool,
        time_from: Duration,
        time_to: Duration,
    ) -> &Event {
        let id = self.next_event_id();
        self.tracker.add(time_from, time_to, id);
        let info = TimerInfo {
            timer_id: self.timers_cnt,
            with_sleep,
            proc,
        };
        let info = EventInfo::TimerInfo(info);
        let event = Event {
            id,
            time_from,
            time_to,
            info,
        };
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

    pub fn get_ready(&self, i: usize) -> Option<&Event> {
        self.tracker.ready(i).and_then(|s| self.get(s.tag))
    }

    pub fn remove_ready(&mut self, i: usize) -> Option<&Event> {
        self.tracker.remove_ready(i).and_then(|s| self.get(s.tag))
    }

    pub fn remove_with_id(&mut self, id: usize) -> Option<&Event> {
        self.tracker
            .remove_with_tag(id)
            .and_then(|s| self.get(s.tag))
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
            .map(|i| self.tracker.ready(i).unwrap().tag)
            .for_each(|i| self.events[i].hash(state));
    }
}
