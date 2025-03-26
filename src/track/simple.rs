use super::{
    segment::{self, Endpoint, Segment},
    tracker::Tracker,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct SimpleTracker<T>
where
    T: Endpoint,
{
    /// Segments are sorted by the left end.
    segments: Vec<Segment<T>>,
}

// ////////////////////////////////////////////////////////////////////////////////

impl<T> SimpleTracker<T>
where
    T: Endpoint,
{
    pub fn new() -> Self {
        Self::default()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Tracker<T> for SimpleTracker<T>
where
    T: Endpoint,
{
    fn add(&mut self, from: T, to: T, tag: usize) {
        let segment = Segment { from, to, tag };
        self.segments.push(segment);
        self.segments.sort();
    }

    fn remove_with_tag(&mut self, tag: usize) -> Option<Segment<T>> {
        self.segments
            .iter()
            .enumerate()
            .find(|(_, s)| s.tag == tag)
            .map(|(index, _)| index)
            .map(|i| self.segments.remove(i))
    }

    fn remove_ready(&mut self, ready: usize) -> Option<Segment<T>> {
        if ready < self.ready_count() {
            Some(self.segments.remove(ready))
        } else {
            None
        }
    }

    fn ready_count(&self) -> usize {
        if self.segments.is_empty() {
            return 0;
        }
        let mut min_to = self.segments[0].to;
        let mut ready = 0;
        while ready < self.segments.len() && self.segments[ready].from <= min_to {
            min_to = min_to.min(self.segments[ready].to);
            ready += 1;
        }
        ready
    }

    fn ready(&self, i: usize) -> Option<&Segment<T>> {
        if i < self.ready_count() {
            Some(&self.segments[i])
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut t = SimpleTracker::<i32>::new();
        t.add(1, 2, 0);
        t.add(1, 3, 1);
        t.add(3, 5, 2);
        assert_eq!(t.ready_count(), 2);
        let remove_result = t.remove_with_tag(0);
        assert!(remove_result.is_some());
        assert_eq!(t.ready_count(), 2);
        let remove_result = t.remove_ready(0);
        assert!(remove_result.is_some());
    }
}
