use std::{cell::RefCell, rc::Rc};

use driver::Driver;

use crate::{event::driver::EventDriver, System, SystemHandle};

pub mod context;
pub mod error;
pub mod hash;
pub mod log;
pub mod net;
pub mod node;
pub mod proc;
pub mod system;

mod driver;

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;

////////////////////////////////////////////////////////////////////////////////

pub struct StepConfig {
    udp_packet_drop_prob: f64, // [0...1]
}

impl StepConfig {
    pub fn new(udp_packet_drop_prob: f64) -> Self {
        Self {
            udp_packet_drop_prob,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Simulation {
    system: System,
    driver: Rc<RefCell<Driver>>,
}

impl Simulation {
    pub fn new(seed: u64) -> Self {
        let driver = Rc::new(RefCell::new(Driver::new(seed)));
        let system = System::new_default_net(&(driver.clone() as Rc<RefCell<dyn EventDriver>>));
        Self { system, driver }
    }

    pub fn step(&self, cfg: &StepConfig) -> bool {
        self.driver
            .borrow_mut()
            .make_step(self.system.handle(), cfg)
    }

    pub fn step_until_no_events(&self, cfg: &StepConfig) {
        loop {
            let step_result = self.step(cfg);
            if !step_result {
                break;
            }
        }
    }

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

    pub fn system(&self) -> SystemHandle {
        self.system.handle()
    }
}
