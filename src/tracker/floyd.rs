use std::{
    collections::BTreeSet,
    fmt::Debug,
    ops::{Add, Neg},
};

use super::{
    graph::{Graph, GraphFloydSmart},
    EventTracker,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
enum LogEntry<T> {
    AddEvent {
        event: usize,
        prev: usize,
        min_time: T,
        max_time: T,
    },
    EventHappen {
        event: usize,
    },
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct FloydEventTracker<T> {
    graph: GraphFloydSmart<T>,
    pending: BTreeSet<usize>,
    log: Vec<LogEntry<T>>,
}

impl<T: Default + Copy> Default for FloydEventTracker<T> {
    fn default() -> Self {
        Self {
            graph: GraphFloydSmart::new(1),
            pending: Default::default(),
            log: Default::default(),
        }
    }
}

impl<T: Default + Copy + Ord + Add<Output = T> + Neg<Output = T>> EventTracker<T>
    for FloydEventTracker<T>
{
    fn add_event(&mut self, prev: usize, min_time: T, max_time: T) -> usize {
        assert!(min_time <= max_time);
        let cur = self.graph.add_vertex();
        self.graph.add_edge(prev, cur, min_time);
        self.graph.add_edge(cur, prev, -max_time);
        self.pending.insert(cur);
        self.log.push(LogEntry::AddEvent {
            event: cur,
            prev,
            min_time,
            max_time,
        });
        cur
    }

    fn event_happen(mut self, event: usize) -> Option<Self> {
        self.log.push(LogEntry::EventHappen { event });
        self.pending
            .iter()
            .filter(|i| **i != event)
            .for_each(|i| self.graph.add_edge(event, *i, T::default()));
        self.validate().map(|mut s| {
            let result = s.pending.remove(&event);
            assert!(result);
            s
        })
    }

    fn pending_events(&self) -> impl Iterator<Item = usize> + '_ {
        self.pending.iter().cloned()
    }

    fn next_events(&self) -> impl Iterator<Item = (usize, Self)> + '_ {
        self.pending_events()
            .filter_map(|e| self.clone().event_happen(e).map(|s| (e, s)))
    }
}

impl<T: Default + Copy + Ord + Add<Output = T> + Neg<Output = T>> FloydEventTracker<T> {
    pub fn add_event(&mut self, prev: usize, min_time: T, max_time: T) -> usize {
        assert!(min_time <= max_time);
        let cur = self.graph.add_vertex();
        self.graph.add_edge(prev, cur, min_time);
        self.graph.add_edge(cur, prev, -max_time);
        self.pending.insert(cur);
        self.log.push(LogEntry::AddEvent {
            event: cur,
            prev,
            min_time,
            max_time,
        });
        cur
    }

    pub fn event_happen(mut self, event: usize) -> Option<Self> {
        self.log.push(LogEntry::EventHappen { event });
        self.pending
            .iter()
            .filter(|i| **i != event)
            .for_each(|i| self.graph.add_edge(event, *i, T::default()));
        self.validate().map(|mut s| {
            let result = s.pending.remove(&event);
            assert!(result);
            s
        })
    }

    pub fn event_time(&self, event: usize) -> (T, T) {
        let (min_time, max_time) = self.event_time_unchecked(event);
        assert!(min_time <= max_time);
        (min_time, max_time)
    }

    fn event_time_unchecked(&self, event: usize) -> (T, T) {
        let min_time = self.graph.max_dist(0, event).unwrap();
        let max_time = -self.graph.max_dist(event, 0).unwrap();
        (min_time, max_time)
    }

    fn validate(self) -> Option<Self> {
        let valid =
            (0..self.graph.size()).all(|i| self.graph.max_dist(i, i).unwrap() == T::default());
        if valid {
            Some(self)
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn validate_full(&self) -> bool {
        (0..self.graph.size()).all(|i| {
            let (min_time, max_time) = self.event_time_unchecked(i);
            min_time <= max_time
        })
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn pending_events(&self) -> impl Iterator<Item = usize> + '_ {
        self.pending.iter().cloned()
    }

    pub fn next_events(&self) -> impl Iterator<Item = (usize, Self)> + '_ {
        self.pending_events()
            .filter_map(|e| self.clone().event_happen(e).map(|s| (e, s)))
    }
}

impl<T: Debug> Debug for FloydEventTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Log:")?;
        for e in self.log.iter() {
            writeln!(f, "{e:?}")?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::FloydEventTracker;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn raft_like_example() {
        let mut t = FloydEventTracker::default();
        assert!(t.validate_full());

        let e1 = t.add_event(
            0,
            Duration::from_millis(250).as_nanos() as i128,
            Duration::from_millis(750).as_nanos() as i128,
        );

        let e2 = t.add_event(
            0,
            Duration::from_millis(250).as_nanos() as i128,
            Duration::from_millis(750).as_nanos() as i128,
        );

        assert!(t.validate_full());

        // e1 happen
        let mut t = t.event_happen(e1).unwrap();
        let e3 = t.add_event(
            e1,
            Duration::from_millis(1).as_nanos() as i128,
            Duration::from_millis(5).as_nanos() as i128,
        );
        let e4 = t.add_event(
            e1,
            Duration::from_millis(1).as_nanos() as i128,
            Duration::from_millis(5).as_nanos() as i128,
        );
        assert!(t.validate_full());

        // 2 happen
        let mut t = t.event_happen(e2).unwrap();
        let e5 = t.add_event(
            e2,
            Duration::from_millis(250).as_nanos() as i128,
            Duration::from_millis(750).as_nanos() as i128,
        );
        assert!(t.validate_full());

        // e3 happen
        let t = t.event_happen(e3).unwrap();
        assert_eq!(t.pending_events().count(), 2);
        assert_eq!(t.next_events().count(), 1);
        assert_eq!(t.next_events().next().map(|(e, _s)| e), Some(e4));
        assert!(t.validate_full());

        // e4 happen
        let t = t.event_happen(e4).unwrap();
        assert_eq!(t.pending_events().count(), 1);
        assert_eq!(t.next_events().count(), 1);
        assert_eq!(t.next_events().next().map(|(e, _s)| e), Some(e5));
        assert!(t.validate_full());

        // e5 happen
        let t = t.event_happen(e5).unwrap();
        assert_eq!(t.pending_events().count(), 0);
        assert_eq!(t.next_events().count(), 0);
        assert!(t.validate_full());

        for e in [e1, e2, e3, e4, e5] {
            println!("time for {e}: {:?}", t.event_time(e));
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic1() {
        let mut t = FloydEventTracker::default();
        let e1 = t.add_event(0, 3, 6);
        let mut t = t.event_happen(e1).unwrap();
        let e2 = t.add_event(e1, 2, 2);
        let e3 = t.add_event(e1, 3, 6);
        let v: Vec<_> = t.next_events().map(|(e, _t)| e).collect();
        assert_eq!(v, [e2]);
        let t = t.event_happen(e2).unwrap();
        assert!(t.validate_full());
        assert!(t.event_happen(e3).unwrap().validate_full());
    }
}
