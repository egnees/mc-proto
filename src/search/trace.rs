use crate::system::sys::System;

use super::{control::Builder, step::Step};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Trace {
    build: Box<dyn Builder>,
    steps: Vec<Step>,
}

impl Trace {
    pub fn new(build: Box<dyn Builder>) -> Self {
        Self {
            build,
            steps: Vec::default(),
        }
    }

    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    pub fn system(&self) -> System {
        let mut sys = self.build.build();
        self.steps.iter().for_each(|s| sys.apply_search_step(s));
        sys
    }
}
