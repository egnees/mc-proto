use std::fmt::{Debug, Display};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Default)]
pub struct SearchLog {
    pub visited_total: usize,
    pub visited_unique: usize,
}

impl SearchLog {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Display for SearchLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unique visited: {}, total visited: {}",
            self.visited_unique, self.visited_total
        )
    }
}

impl Debug for SearchLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
