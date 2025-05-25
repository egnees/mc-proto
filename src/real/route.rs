use std::{collections::HashMap, net::SocketAddr};

use crate::Address;

////////////////////////////////////////////////////////////////////////////////

/// Represents mapping from the process address to its real
/// listening address.
#[derive(Clone, Default, Debug)]
pub struct RouteConfig {
    addrs: HashMap<Address, SocketAddr>,
}

impl RouteConfig {
    /// Create new config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entry in the config.
    pub fn add(
        &mut self,
        proc: impl Into<Address>,
        addr: impl Into<SocketAddr>,
    ) -> Option<SocketAddr> {
        self.addrs.insert(proc.into(), addr.into())
    }

    pub(crate) fn get(&self, proc: &Address) -> Option<&SocketAddr> {
        self.addrs.get(proc)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Comfortable builder for the route config [`RouteConfig`].
#[derive(Default)]
pub struct RouteConfigBuilder {
    cfg: RouteConfig,
}

impl RouteConfigBuilder {
    /// Make new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entry
    pub fn add(self, proc: impl Into<Address>, addr: impl Into<SocketAddr>) -> Self {
        let mut builder = Self { cfg: self.cfg };
        builder.cfg.add(proc, addr);
        builder
    }

    /// Build config.
    pub fn build(self) -> RouteConfig {
        self.cfg
    }
}
