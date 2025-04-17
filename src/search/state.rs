use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{event::driver::EventDriver, System};

use super::{gen::Generator, step::StateTraceStep};

////////////////////////////////////////////////////////////////////////////////

pub struct SearchState {
    pub(crate) system: System,
    pub(crate) gen: Rc<RefCell<Generator>>,
}

impl SearchState {
    pub fn from_trace(trace: &StateTrace) -> Self {
        let gen = Rc::new(RefCell::new(Generator::new()));
        let driver = gen.clone() as Rc<RefCell<dyn EventDriver>>;
        let system = System::new_default_net(&driver);
        let mut state = Self { system, gen };
        trace.apply_steps(&mut state);
        state
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Default)]
pub struct StateTrace {
    steps: Vec<StateTraceStep>,
}

impl StateTrace {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_step(&mut self, step: StateTraceStep) {
        self.steps.push(step);
    }

    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    pub fn step(&self, i: usize) -> &StateTraceStep {
        self.steps.get(i).unwrap()
    }

    fn apply_steps(&self, state: &mut SearchState) {
        self.steps.iter().for_each(|step| step.apply(state));
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
