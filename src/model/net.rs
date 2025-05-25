use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    time::Duration,
};

use super::{context::Context, error::Error};
use crate::Address;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct NetworkState {
    pub min_packet_delay: Duration,
    pub max_packet_delay: Duration,
}

impl NetworkState {
    pub fn new(cfg: &Config) -> Self {
        Self {
            min_packet_delay: cfg.min_packet_delay,
            max_packet_delay: cfg.max_packet_delay,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Network(Rc<RefCell<NetworkState>>);

impl Network {
    pub fn new(cfg: &Config) -> Self {
        Self(Rc::new(RefCell::new(NetworkState::new(cfg))))
    }

    pub fn handle(&self) -> NetworkHandle {
        NetworkHandle(Rc::downgrade(&self.0))
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct NetworkHandle(Weak<RefCell<NetworkState>>);

impl NetworkHandle {
    fn state(&self) -> Rc<RefCell<NetworkState>> {
        self.0.upgrade().unwrap()
    }

    pub fn set_delays(&self, min: Duration, max: Duration) -> Result<(), Error> {
        if min > max {
            Err(Error::IncorrectRange)
        } else {
            let state = self.state();
            let mut state = state.borrow_mut();
            state.min_packet_delay = min;
            state.max_packet_delay = max;
            Ok(())
        }
    }

    pub fn delays_range(&self) -> (Duration, Duration) {
        let state = self.state();
        let state = state.borrow();
        (state.min_packet_delay, state.max_packet_delay)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Config {
    pub min_packet_delay: Duration,
    pub max_packet_delay: Duration,
}

impl Config {
    pub fn new(min_packet_delay: Duration, max_packet_delay: Duration) -> Result<Self, Error> {
        // let min_packet_delay = time_from_f64(min_packet_delay)?;
        // let max_packet_delay = time_from_f64(max_packet_delay)?;
        if min_packet_delay > max_packet_delay {
            Err(Error::IncorrectRange)
        } else {
            Ok(Config {
                min_packet_delay,
                max_packet_delay,
            })
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to send UDP message.
pub fn send_message(to: &Address, content: impl Into<String>) {
    Context::current().register_udp_message(to, content.into());
}

////////////////////////////////////////////////////////////////////////////////
