use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, Sub},
    time::Duration,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, PartialOrd)]
pub enum Time {
    Point(Duration),
    Segment(TimeSegment),
}

impl Time {
    pub fn default_range() -> Self {
        Self::Segment(TimeSegment::default())
    }

    pub fn default_point() -> Self {
        Self::Point(Duration::default())
    }

    pub fn new_segment(from: Duration, to: Duration) -> Self {
        Self::Segment(TimeSegment::new(from, to))
    }

    pub fn new_point(t: Duration) -> Self {
        Self::Point(t)
    }

    pub fn shift(&self, value: Duration) -> Self {
        let mut c = *self;
        match &mut c {
            Time::Point(duration) => *duration = duration.add(value),
            Time::Segment(time_segment) => *time_segment = time_segment.shift(value),
        };
        c
    }

    pub fn shift_neg(&self, value: Duration) -> Self {
        let mut c = *self;
        match &mut c {
            Time::Point(duration) => *duration = duration.sub(value),
            Time::Segment(time_segment) => {
                *time_segment = time_segment.shift_neg(value);
            }
        };
        c
    }

    pub fn shift_range(&self, from: Duration, to: Duration) -> Self {
        let mut c = *self;
        match &mut c {
            Time::Point(duration) => {
                return Self::Segment(TimeSegment::new(from, to).shift(*duration));
            }
            Time::Segment(time_segment) => *time_segment = time_segment.shift_range(from, to),
        }
        c
    }

    pub fn shift_on(&self, t: Time) -> Time {
        match t {
            Time::Point(duration) => self.shift(duration),
            Time::Segment(time_segment) => self.shift_range(time_segment.from, time_segment.to),
        }
    }

    pub fn min(&self) -> Duration {
        match self {
            Time::Point(duration) => *duration,
            Time::Segment(time_segment) => time_segment.from,
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Time::Point(duration) => write!(f, "[{:.3}]", duration.as_secs_f64()),
            Time::Segment(time_segment) => write!(f, "{}", time_segment),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Hash, Default, PartialEq, Eq, Copy)]
pub struct TimeSegment {
    pub from: Duration,
    pub to: Duration,
}

impl TimeSegment {
    pub fn new(from: Duration, to: Duration) -> Self {
        assert!(from <= to);
        Self { from, to }
    }

    pub fn shift(&self, value: Duration) -> Self {
        self.shift_range(value, value)
    }

    pub fn shift_range(&self, from: Duration, to: Duration) -> Self {
        assert!(from <= to);
        TimeSegment::new(self.from + from, self.to + to)
    }

    pub fn shift_neg(&self, on: Duration) -> Self {
        TimeSegment::new(self.from - on, self.to - on)
    }
}

impl Display for TimeSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:.3} - {:.3}]",
            self.from.as_secs_f64(),
            self.to.as_secs_f64()
        )
    }
}

impl PartialOrd for TimeSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.from == other.from && self.to == other.to {
            Some(Ordering::Equal)
        } else if self.from <= other.from && self.to <= other.to {
            Some(Ordering::Less)
        } else if other.from <= self.from && other.to <= self.to {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}
