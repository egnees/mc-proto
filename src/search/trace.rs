use std::fmt::{Debug, Display};

use crate::system::sys::System;

use super::{control::Builder, step::StateTraceStep};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StateTrace {
    build: Box<dyn Builder>,
    steps: Vec<StateTraceStep>,
}

impl StateTrace {
    pub fn new(build: Box<dyn Builder>) -> Self {
        Self {
            build,
            steps: Vec::default(),
        }
    }

    pub fn add_step(&mut self, step: StateTraceStep) {
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

    pub fn step(&self, i: usize) -> &StateTraceStep {
        self.steps.get(i).unwrap()
    }
}

impl Debug for StateTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Trace").field("steps", &self.steps).finish()
    }
}

impl Display for StateTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in self.steps.iter() {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}
