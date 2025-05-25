use std::fmt::{Debug, Display};

////////////////////////////////////////////////////////////////////////////////

/// Represents log of the search,
/// which is returned after search is complete.
#[derive(Clone, Default)]
pub struct SearchLog {
    /// Total number of states, visited during the search
    pub visited_total: usize,

    /// Total number of uniques states with different hash,
    /// visited during the search.
    pub visited_unique: usize,
}

impl SearchLog {
    pub(crate) fn new() -> Self {
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
