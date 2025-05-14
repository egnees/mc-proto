use super::{term::Term, vote::VotedFor};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Common {
    current_term: Term,
    voted_for: VotedFor,
    commit_index: u64,
    last_applied: u64,
}

////////////////////////////////////////////////////////////////////////////////

pub struct Leader {
    next_index: Vec<u64>,
    match_index: Vec<u64>,
}

impl Leader {
    pub fn new(nodes: usize, last_log_index: u64) -> Self {
        Self {
            next_index: vec![last_log_index + 1; nodes],
            match_index: vec![0; nodes],
        }
    }
}
