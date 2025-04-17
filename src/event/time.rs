use std::{fmt::Display, time::Duration};

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

    pub fn shift(&self, value: Duration) -> TimeSegment {
        self.shift_range(value, value)
    }

    pub fn shift_range(&self, from: Duration, to: Duration) -> TimeSegment {
        TimeSegment::new(self.from + from, self.to + to)
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
