//! Allow to send network messages and asyncronously wait for the response.

use serde::{Deserialize, Serialize};

use crate::{model, real};

use thiserror::Error;

use super::{mode::is_real, Address};

////////////////////////////////////////////////////////////////////////////////

/// Represents RPC request.
pub enum RpcRequest {
    /// Real RPC request
    Real(real::RpcRequest),

    /// Model of RPC request.
    Model(model::RpcRequest),
}

impl From<real::RpcRequest> for RpcRequest {
    fn from(value: real::RpcRequest) -> Self {
        Self::Real(value)
    }
}

impl From<model::RpcRequest> for RpcRequest {
    fn from(value: model::RpcRequest) -> Self {
        Self::Model(value)
    }
}

impl RpcRequest {
    /// Allow sto unpack value from the serialized request.
    pub fn unpack<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        match self {
            RpcRequest::Real(real) => real.unpack(),
            RpcRequest::Model(sim) => sim.unpack().ok(),
        }
    }

    /// Allows to reply of the request with provided value.
    /// The value will be serialized and sent over the network.
    pub fn reply<T: Serialize>(self, value: &T) -> RpcResult<()> {
        match self {
            RpcRequest::Real(real) => real.reply(value),
            RpcRequest::Model(sim) => sim.reply(value),
        }
    }

    /// Allows to get tag of the request,
    /// which can be useful when type of the request
    /// is not known.
    pub fn tag(&self) -> u64 {
        match self {
            RpcRequest::Real(real) => real.tag,
            RpcRequest::Model(sim) => sim.tag,
        }
    }

    /// Allows to get request sender address.
    pub fn from(&self) -> &Address {
        match self {
            RpcRequest::Real(real) => &real.from,
            RpcRequest::Model(sim) => sim.from(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents RPC response.
pub enum RpcResponse {
    /// Real RPC response.
    Real(real::RpcResponse),

    /// Model of RPC response.
    Model(model::RpcResponse),
}

impl From<real::RpcResponse> for RpcResponse {
    fn from(value: real::RpcResponse) -> Self {
        Self::Real(value)
    }
}

impl From<model::RpcResponse> for RpcResponse {
    fn from(value: model::RpcResponse) -> Self {
        Self::Model(value)
    }
}

impl RpcResponse {
    /// Allow to unpack value of specified type from the RPC response.
    pub fn unpack<'a, T: Deserialize<'a>>(&'a self) -> Option<T> {
        match self {
            RpcResponse::Real(real) => real.unpack(),
            RpcResponse::Model(sim) => sim.unpack().ok(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents error which can happen during the RPC request.
#[derive(Error, Debug, Clone, Hash)]
pub enum RpcError {
    /// Some internal error.
    #[error("internal: {info}")]
    Internal {
        /// Describes internal error.
        info: String,
    },

    /// Got when trying to register the second RPC listener [`crate::RpcListener`]
    /// from the same address.
    #[error("already listening for rpc requests")]
    AlreadyListening,

    /// Returned when connection is refused.
    #[error("connection refused")]
    ConnectionRefused,
    #[error("not found")]

    /// If process with specified address is not found
    NotFound,
    #[error("address not resolved")]

    /// If address of process is not resolved in real mode (see [`crate::real::RouteConfig`]).
    AddressNotResolved,
}

impl From<serde_json::Error> for RpcError {
    fn from(value: serde_json::Error) -> Self {
        Self::Internal {
            info: value.to_string(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents result of RPC interaction
pub type RpcResult<T> = Result<T, RpcError>;

////////////////////////////////////////////////////////////////////////////////

/// Allows to send RCP to the specifeid process with specified tag and content.
/// Returns result with response from the receiver or error.
pub async fn rpc<T: Serialize>(to: Address, tag: u64, value: &T) -> RpcResult<RpcResponse> {
    if is_real() {
        real::rpc(to, tag, value).await.map(RpcResponse::from)
    } else {
        model::rpc(to, tag, value).await.map(RpcResponse::from)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents listener of the RPC requests.
pub enum RpcListener {
    /// Real listener
    Real(real::RpcListener),

    /// Model of listener
    Model(model::RpcListener),
}

impl From<real::RpcListener> for RpcListener {
    fn from(value: real::RpcListener) -> Self {
        Self::Real(value)
    }
}

impl From<model::RpcListener> for RpcListener {
    fn from(value: model::RpcListener) -> Self {
        Self::Model(value)
    }
}

impl RpcListener {
    /// Allows to listen for RPC requests.
    /// Returns when some RPC request is received.
    /// Caches the received requests after register [`RpcListener::register`] is called.
    pub async fn listen(&mut self) -> RpcRequest {
        match self {
            RpcListener::Real(real) => real.listen().await.into(),
            RpcListener::Model(sim) => sim.listen().await.into(),
        }
    }

    /// Allows to register RPC listener.
    /// No two listeners can be registered from the same process in the same time.
    /// After that, all receiving requests will be stored in memory and can be responsed
    /// after listen [`RpcListener::listen`] calls.
    pub fn register() -> RpcResult<Self> {
        if is_real() {
            real::context::Context::current()
                .register_rpc_listener()
                .map(RpcListener::from)
        } else {
            model::RpcListener::register().map(RpcListener::from)
        }
    }
}
