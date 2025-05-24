use std::{collections::HashMap, net::SocketAddr};

use crate::Address;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Default)]
pub struct RouteConfig {
    addrs: HashMap<Address, SocketAddr>,
}

impl RouteConfig {
    pub fn new() -> Self {
        Self::default()
    }

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

#[derive(Default)]
pub struct RouteConfigBuilder {
    cfg: RouteConfig,
}

impl RouteConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(self, proc: impl Into<Address>, addr: impl Into<SocketAddr>) -> Self {
        let mut builder = Self { cfg: self.cfg };
        builder.cfg.add(proc, addr);
        builder
    }

    pub fn build(self) -> RouteConfig {
        self.cfg
    }
}
