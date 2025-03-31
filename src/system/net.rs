use std::time::Duration;

use super::{context::Context, error::Error, proc::Address};

////////////////////////////////////////////////////////////////////////////////

pub struct Network {
    pub min_packet_delay: Duration,
    pub max_packet_delay: Duration,
}

impl Network {
    pub fn new(cfg: &Config) -> Self {
        Self {
            min_packet_delay: cfg.min_packet_delay,
            max_packet_delay: cfg.max_packet_delay,
        }
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

////////////////////////////////////////////////////////////////////////////////

pub fn send_message(to: &Address, content: impl Into<String>) {
    Context::current().register_udp_message(to, content.into());
}

////////////////////////////////////////////////////////////////////////////////
