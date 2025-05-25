use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    mc::error::SearchErrorKind, mc::SearchConfig, model::event::driver::EventDriver,
    model::system::System, model::SystemHandle,
};

use super::{gen::Generator, step::StateTraceStep};

////////////////////////////////////////////////////////////////////////////////

pub struct SearchState {
    pub(crate) system: System,
    pub(crate) gen: Rc<RefCell<Generator>>,
}

impl SearchState {
    pub fn from_trace(trace: &StateTrace) -> Result<Self, SearchErrorKind> {
        let gen = Rc::new(RefCell::new(Generator::new()));
        let driver = gen.clone() as Rc<RefCell<dyn EventDriver>>;
        let system = System::new_default_net(&driver);
        let mut state = Self { system, gen };
        trace.apply_steps(&mut state)?;
        Ok(state)
    }

    pub fn steps(&self, cfg: &SearchConfig) -> Vec<StateTraceStep> {
        let system = self.system.handle();
        self.gen.borrow().steps(system, cfg)
    }

    pub fn select_ready_event(&self, i: usize) {
        self.gen.borrow_mut().select_ready_event(i);
        self.system.handle().run_async_tasks();
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
            let mut apply_result = step.apply(state);
            if let Err(SearchErrorKind::ProcessPanic(p)) = apply_result.as_mut() {
                let steps = self.steps.as_slice()[..i + 1].to_vec();
                let trace = StateTrace { steps };
                p.trace = Some(trace);
            }
            apply_result?;
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

/// Represents view of the search state,
/// which includes state trace (which led to the state)
/// and corresponding model of the system.
#[derive(Clone)]
pub struct StateView {
    system: SystemHandle,
    trace: StateTrace,
}

impl StateView {
    pub(crate) fn new(state: &SearchState, trace: StateTrace) -> Self {
        Self {
            system: state.system.handle(),
            trace,
        }
    }

    /// Get system model corresponding to the search state.
    pub fn system(&self) -> SystemHandle {
        self.system.clone()
    }

    /// Get depth of the trace led to state.
    pub fn depth(&self) -> usize {
        self.trace.steps.len()
    }
}
