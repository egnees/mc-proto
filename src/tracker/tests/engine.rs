use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::tracker::EventTracker;

////////////////////////////////////////////////////////////////////////////////

pub trait Engine {
    fn select(&mut self, n: usize) -> usize;

    fn time(&mut self) -> (i64, i64);

    fn next_events(&mut self) -> usize;

    fn add_events(&mut self, prev: usize, t: &mut impl EventTracker<i64>) {
        for _ in 0..self.next_events() {
            let (min_time, max_time) = self.time();
            t.add_event(prev, min_time, max_time);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct RandomTimeEngine {
    max_events: usize,
    rng: StdRng,
    ranges: Vec<(i64, i64)>, // (start, len)
}

impl RandomTimeEngine {
    pub fn new(seed: u64, max_events: usize, ranges: impl Iterator<Item = (i64, i64)>) -> Self {
        Self {
            max_events,
            rng: StdRng::seed_from_u64(seed),
            ranges: Vec::from_iter(ranges),
        }
    }
}

impl Engine for RandomTimeEngine {
    fn select(&mut self, n: usize) -> usize {
        self.rng.random_range(0..n)
    }

    fn time(&mut self) -> (i64, i64) {
        let range = self.select(self.ranges.len());
        let (start, len) = self.ranges[range];
        let l = self.rng.random_range(start..start + len);
        let r = self.rng.random_range(start..start + len);
        if l <= r {
            (l, r)
        } else {
            (r, l)
        }
    }

    fn next_events(&mut self) -> usize {
        self.rng.random_range(0..self.max_events)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct FixedTimeEngine {
    max_events: usize,
    rng: StdRng,
    ranges: Vec<(i64, i64)>, // (start, len)
}

impl FixedTimeEngine {
    pub fn new(seed: u64, max_events: usize, ranges: impl Iterator<Item = (i64, i64)>) -> Self {
        Self {
            max_events,
            rng: StdRng::seed_from_u64(seed),
            ranges: Vec::from_iter(ranges),
        }
    }
}

impl Engine for FixedTimeEngine {
    fn select(&mut self, n: usize) -> usize {
        self.rng.random_range(0..n)
    }

    fn time(&mut self) -> (i64, i64) {
        let range = self.select(self.ranges.len());
        let (start, len) = self.ranges[range];
        (start, start + len)
    }

    fn next_events(&mut self) -> usize {
        self.rng.random_range(0..self.max_events)
    }
}
