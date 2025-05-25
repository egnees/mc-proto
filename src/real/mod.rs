//! Provides real mode,
//! which allows to run system in real environment.

pub(crate) mod context;
mod error;
mod fs;
mod join;
mod node;
mod proc;
mod route;
mod rpc;
mod timer;

////////////////////////////////////////////////////////////////////////////////

pub use error::Error;
pub(crate) use fs::file::File;
pub(crate) use join::JoinHandle;
pub use node::RealNode;
pub use proc::{LocalReceiver, LocalSender};
pub use route::{RouteConfig, RouteConfigBuilder};
pub(crate) use rpc::{rpc, RpcListener, RpcRequest, RpcResponse};
pub(crate) use timer::Timer;

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
