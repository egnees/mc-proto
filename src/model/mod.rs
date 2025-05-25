//! This module provide model of system,
//! which can be used in deterministic simulation or model checking.

pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod event;
pub(crate) mod fs;
pub(crate) mod hash;
pub(crate) mod log;
pub(crate) mod net;
pub(crate) mod node;
pub(crate) mod proc;
pub(crate) mod rpc;
pub(crate) mod runtime;
pub(crate) mod system;
pub(crate) mod tcp;
pub(crate) mod timer;

pub(crate) use fs::file::File;

pub(crate) use rpc::{rpc, RpcListener, RpcRequest, RpcResponse};

pub use system::SystemHandle;

pub use log::{Log, LogEntry};
pub(crate) use runtime::JoinHandle;
pub(crate) use timer::Timer;

pub use net::send_message;

pub use node::Node;

pub use system::HashType;

pub use tcp::{listen::TcpListener, stream::TcpReceiver, stream::TcpStream, TcpError, TcpSender};

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
