use std::{cell::RefCell, rc::Rc};

use crate::model::{event::driver::EventDriver, system::System, SystemHandle};

use super::driver::Driver;

////////////////////////////////////////////////////////////////////////////////

/// Specifies policy of making single simulation step.
pub struct StepConfig {
    /// Specifies probability of udp packet drop.
    pub udp_packet_drop_prob: f64, // [0...1]
}

impl StepConfig {
    /// Allows to make new step config.
    pub fn new(udp_packet_drop_prob: f64) -> Self {
        Self {
            udp_packet_drop_prob,
        }
    }

    /// Allows to make step config with zero drop probability.
    pub fn no_drops() -> Self {
        Self::new(0.)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents deterministic simulation
pub struct Simulation {
    system: System,
    driver: Rc<RefCell<Driver>>,
}

impl Simulation {
    /// Allows to create simulation with specified seed.
    pub fn new(seed: u64) -> Self {
        let driver = Rc::new(RefCell::new(Driver::new(seed)));
        let system = System::new_default_net(&(driver.clone() as Rc<RefCell<dyn EventDriver>>));
        Self { system, driver }
    }

    /// Allows to make single simulation step.
    /// The UDP packets will be dropped with probability,
    /// specified in `cfg` [StepConfig::udp_packet_drop_prob].
    pub fn step(&self, cfg: &StepConfig) -> bool {
        let outcome = self.driver.borrow_mut().next_event_outcome(cfg);
        if let Some(outcome) = outcome {
            self.system.handle().handle_event_outcome(outcome);
            true
        } else {
            false
        }
    }

    /// Allows to make steps until there are no events.
    pub fn step_until_no_events(&self, cfg: &StepConfig) {
        loop {
            let step_result = self.step(cfg);
            if !step_result {
                break;
            }
        }
    }

    /// Allows to make steps until provided predicate is true.
    /// Returns `false` if there are no events, but predicate is still false.
    pub fn step_unti<F>(&self, mut f: F, cfg: &StepConfig) -> bool
    where
        F: FnMut(SystemHandle) -> bool,
    {
        loop {
            if f(self.system.handle()) {
                return true;
            }
            let step_result = self.step(cfg);
            if !step_result {
                return false;
            }
        }
    }

    /// Returns handle on the system model [`crate::model::SystemHandle`].
    pub fn system(&self) -> SystemHandle {
        self.system.handle()
    }
}
