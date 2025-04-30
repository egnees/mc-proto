use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{event::driver::EventDriver, SearchErrorKind, System, SystemHandle};

use super::{gen::Generator, step::StateTraceStep};

////////////////////////////////////////////////////////////////////////////////

pub struct SearchState {
    pub(crate) system: System,
    pub(crate) gen: Rc<RefCell<Generator>>,
    pub(crate) trace_depth: usize,
}

impl SearchState {
    pub fn from_trace(trace: &StateTrace) -> Result<Self, SearchErrorKind> {
        let mut state = SearchState::default();
        trace.apply_steps(&mut state)?;
        Ok(state)
    }

    pub fn system(&self) -> SystemHandle {
        self.system.handle()
    }

    pub fn depth(&self) -> usize {
        self.trace_depth
    }

    pub fn view(&self) -> StateView {
        StateView {
            system: self.system.handle(),
            trace_depth: self.trace_depth,
        }
    }

    pub(crate) fn apply_step(&mut self, step: &StateTraceStep) -> Result<(), SearchErrorKind> {
        step.apply(self)
    }
}

impl Default for SearchState {
    fn default() -> Self {
        let gen = Rc::new(RefCell::new(Generator::new()));
        let driver = gen.clone() as Rc<RefCell<dyn EventDriver>>;
        let system = System::new_default_net(&driver);
        Self {
            system,
            gen,
            trace_depth: 0,
        }
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

    fn apply_steps(&self, state: &mut SearchState) -> Result<(), SearchErrorKind> {
        for i in 0..self.steps.len() {
            let step = &self.steps[i];
            step.apply(state)?;
        }
        Ok(())
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

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StateView {
    system: SystemHandle,
    trace_depth: usize,
}

impl StateView {
    #[allow(unused)]
    pub(crate) fn new(state: &SearchState, trace_depth: usize) -> Self {
        Self {
            system: state.system.handle(),
            trace_depth,
        }
    }

    pub fn system(&self) -> SystemHandle {
        self.system.clone()
    }

    pub fn depth(&self) -> usize {
        self.trace_depth
    }
}
