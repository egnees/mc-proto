use std::collections::HashMap;

use crate::HashType;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Graph {
    pub adj: HashMap<HashType, Vec<HashType>>,
}

impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, from: HashType, to: HashType) {
        self.adj.entry(from).or_default().push(to);
    }

    pub fn sort(&mut self) {
        for (_, v) in self.adj.iter_mut() {
            v.sort();
        }
    }
}
