//! Provides deterministic simulation over the system model [`crate::model`].

mod driver;
mod sim;

pub use sim::{Simulation, StepConfig};

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
