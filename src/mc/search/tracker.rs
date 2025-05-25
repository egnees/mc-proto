pub trait Endpoint: Ord + Copy + Default {}

////////////////////////////////////////////////////////////////////////////////

impl<T> Endpoint for T where T: Ord + Copy + Default {}

////////////////////////////////////////////////////////////////////////////////

/// Represents segment with custom type of endpoints.
/// Segments can be customized with [tags](Segment::tag).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct EventTimespan<T>
where
    T: Endpoint,
{
    /// Min possible time for event.
    pub from: T,

    /// Max possible time for event.
    pub to: T,

    /// Custom tag.
    pub event_id: usize,
}

////////////////////////////////////////////////////////////////////////////////

impl<T> EventTimespan<T>
where
    T: Endpoint,
{
    pub fn new(from: T, to: T, tag: usize) -> Self {
        Self {
            from,
            to,
            event_id: tag,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Tracker<T: Endpoint> {
    segments: Vec<EventTimespan<T>>,
}

impl<T: Endpoint> Tracker<T> {
    pub fn add(&mut self, from: T, to: T, id: usize) {
        let segment = EventTimespan::new(from, to, id);
        self.segments.push(segment);
        self.segments.sort();
    }

    pub fn ready_count(&self) -> usize {
        let Some(r) = self.min_right() else {
            return 0;
        };
        let mut cnt = 0;
        while cnt < self.segments.len() && self.segments[cnt].from <= r {
            cnt += 1;
        }
        cnt
    }

    pub fn get_ready(&self, i: usize) -> Option<EventTimespan<T>> {
        assert!(i < self.ready_count());
        let to = self.min_right()?;
        self.segments.get(i).cloned().map(|s| EventTimespan {
            from: s.from,
            to: s.to.min(to),
            event_id: s.event_id,
        })
    }

    pub fn remove_ready(&mut self, i: usize) -> Option<EventTimespan<T>> {
        assert!(i < self.ready_count());
        let to = self.min_right()?;

        let mut seg = self.segments.remove(i);
        seg.to = seg.to.min(to);
        assert!(seg.from <= seg.to);

        let from = seg.from;
        for s in self.segments.iter_mut() {
            s.from = s.from.max(from);
            assert!(s.from <= s.to);
        }

        self.segments.sort();

        Some(seg)
    }

    pub fn remove_by_event_id(&mut self, id: usize) -> Option<EventTimespan<T>> {
        for i in 0..self.segments.len() {
            if self.segments[i].event_id == id {
                let removed = self.segments.remove(i);
                return Some(removed);
            }
        }
        None
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn min_right(&self) -> Option<T> {
        let mut r = self.segments.first()?.to;
        for s in self.segments.iter() {
            r = r.min(s.to);
        }
        Some(r)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ord_segments() {
        let a = EventTimespan::new(1, 2, 0);
        let b = EventTimespan::new(1, 2, 1);
        let c = EventTimespan::new(1, 3, 0);
        let d = EventTimespan::new(2, 2, 0);
        let segments = [a, b, c, d];
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!(segments[i] < segments[j]);
            }
        }
    }

    #[test]
    fn basic() {
        let mut t = Tracker::<i32>::default();
        t.add(1, 2, 0);
        t.add(1, 3, 1);
        t.add(3, 5, 2);
        assert_eq!(t.ready_count(), 2);
        let remove_result = t.remove_ready(0);
        assert!(remove_result.is_some());
        assert_eq!(t.ready_count(), 2);
        let remove_result = t.remove_ready(0);
        assert!(remove_result.is_some());
    }
}
