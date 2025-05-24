use serde::{Deserialize, Serialize};

use crate::real;

use thiserror::Error;

use super::{mode::is_real, Address};

////////////////////////////////////////////////////////////////////////////////

pub enum RpcRequest {
    Real(real::RpcRequest),
    Sim(crate::rpc::RpcRequest),
}

impl From<real::RpcRequest> for RpcRequest {
    fn from(value: real::RpcRequest) -> Self {
        Self::Real(value)
    }
}

impl From<crate::rpc::RpcRequest> for RpcRequest {
    fn from(value: crate::rpc::RpcRequest) -> Self {
        Self::Sim(value)
    }
}

impl RpcRequest {
    pub fn unpack<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        match self {
            RpcRequest::Real(real) => real.unpack(),
            RpcRequest::Sim(sim) => sim.unpack().ok(),
        }
    }

    pub fn reply<T: Serialize>(self, value: &T) -> RpcResult<()> {
        match self {
            RpcRequest::Real(real) => real.reply(value),
            RpcRequest::Sim(sim) => sim.reply(value),
        }
    }

    pub fn tag(&self) -> u64 {
        match self {
            RpcRequest::Real(real) => real.tag,
            RpcRequest::Sim(sim) => sim.tag,
        }
    }

    pub fn from(&self) -> &Address {
        match self {
            RpcRequest::Real(real) => &real.from,
            RpcRequest::Sim(sim) => sim.from(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum RpcResponse {
    Real(real::RpcResponse),
    Sim(crate::rpc::RpcResponse),
}

impl From<real::RpcResponse> for RpcResponse {
    fn from(value: real::RpcResponse) -> Self {
        Self::Real(value)
    }
}

impl From<crate::rpc::RpcResponse> for RpcResponse {
    fn from(value: crate::rpc::RpcResponse) -> Self {
        Self::Sim(value)
    }
}

impl RpcResponse {
    pub fn unpack<'a, T: Deserialize<'a>>(&'a self) -> Option<T> {
        match self {
            RpcResponse::Real(real) => real.unpack(),
            RpcResponse::Sim(sim) => sim.unpack().ok(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone, Hash)]
pub enum RpcError {
    #[error("internal: {info}")]
    Internal { info: String },
    #[error("already listening for rpc requests")]
    AlreadyListening,
    #[error("connection refused")]
    ConnectionRefused,
    #[error("not found")]
    NotFound,
    #[error("address not resolved")]
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

pub type RpcResult<T> = Result<T, RpcError>;

////////////////////////////////////////////////////////////////////////////////

pub async fn rpc<T: Serialize>(to: Address, tag: u64, value: &T) -> RpcResult<RpcResponse> {
    if is_real() {
        real::rpc(to, tag, value).await.map(RpcResponse::from)
    } else {
        crate::rpc::rpc(to, tag, value).await.map(RpcResponse::from)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum RpcListener {
    Real(real::RpcListener),
    Sim(crate::rpc::RpcListener),
}

impl From<real::RpcListener> for RpcListener {
    fn from(value: real::RpcListener) -> Self {
        Self::Real(value)
    }
}

impl From<crate::rpc::RpcListener> for RpcListener {
    fn from(value: crate::rpc::RpcListener) -> Self {
        Self::Sim(value)
    }
}

impl RpcListener {
    pub async fn listen(&mut self) -> RpcRequest {
        match self {
            RpcListener::Real(real) => real.listen().await.into(),
            RpcListener::Sim(sim) => sim.listen().await.into(),
        }
    }

    pub fn register() -> RpcResult<Self> {
        if is_real() {
            real::context::Context::current()
                .register_rpc_listener()
                .map(RpcListener::from)
        } else {
            crate::rpc::RpcListener::register().map(RpcListener::from)
        }
    }
}
