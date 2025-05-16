use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
    ops::{Add, Neg},
};

use super::{graph::MooreGraph, EventTracker};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct MooreEventTracker<T> {
    g: Option<MooreGraph<T>>,
    pending: BTreeSet<usize>,
}

impl<T: Default> Default for MooreEventTracker<T> {
    fn default() -> Self {
        Self {
            g: Some(Default::default()),
            pending: Default::default(),
        }
    }
}

impl<T> EventTracker<T> for MooreEventTracker<T>
where
    T: Default + Copy + Ord + Add<Output = T> + Neg<Output = T>,
{
    fn add_event(&mut self, prev: usize, min_time: T, max_time: T) -> usize {
        let mut g = self.g.take().unwrap();
        let v = g.add_vertex();
        let g = g
            .add_edge(prev, v, min_time)
            .unwrap()
            .add_edge(v, prev, -max_time)
            .unwrap();
        self.g = Some(g);
        self.pending.insert(v);
        v
    }

    fn event_happen(self, event: usize) -> Option<Self> {
        let mut pending = self.pending;
        let result = pending.remove(&event);
        assert!(result);

        let mut g = self.g.unwrap();
        for pending in pending.iter() {
            g = g.add_edge(event, *pending, T::default())?;
        }
        Some(Self {
            g: Some(g),
            pending,
        })
    }

    fn pending_events(&self) -> impl Iterator<Item = usize> + '_ {
        self.pending.iter().cloned()
    }

    fn next_events(&self) -> impl Iterator<Item = (usize, Self)> + '_ {
        self.pending_events()
            .filter_map(|e| self.clone().event_happen(e).map(|t| (e, t)))
    }
}

impl<T> MooreEventTracker<T>
where
    T: Default + Copy + Ord + Add<Output = T> + Neg<Output = T>,
{
    pub fn cancel_event(&mut self, event: usize) {
        let result = self.pending.remove(&event);
        assert!(result);
    }

    pub fn event_time(&self, event: usize) -> T {
        self.g.as_ref().unwrap().dist(event).unwrap()
    }
}

impl<T> MooreEventTracker<T>
where
    T: Default + Copy + Ord + Add<Output = T> + Neg<Output = T> + Hash,
{
    pub fn hash_pending(&self, hasher: &mut impl Hasher) {
        let min_time = self
            .pending_events()
            .map(|e| self.event_time(e))
            .min()
            .unwrap_or(T::default());
        let mut events: Vec<_> = self
            .pending_events()
            .map(|e| self.event_time(e) + (-min_time))
            .collect();
        events.sort();
        events.iter().for_each(|t| t.hash(hasher));
    }
}
