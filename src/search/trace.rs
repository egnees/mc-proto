use std::fmt::{Debug, Display};

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

    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    pub fn step(&self, i: usize) -> &Step {
        self.steps.get(i).unwrap()
    }
}

impl Debug for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Trace").field("steps", &self.steps).finish()
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in self.steps.iter() {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}
