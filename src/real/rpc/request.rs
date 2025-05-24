use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{
    real::{self, context::Context},
    Address, RpcError, RpcResult,
};

use super::{response::RpcResponse, server::real_rpc::rpc_client::RpcClient};

////////////////////////////////////////////////////////////////////////////////

pub mod real_rpc {
    tonic::include_proto!("real_rpc");
}

////////////////////////////////////////////////////////////////////////////////

pub struct RpcRequest {
    pub(crate) from: Address,
    pub(crate) tag: u64,
    pub(crate) content: Vec<u8>,
    pub(crate) resp: oneshot::Sender<Vec<u8>>,
}

impl RpcRequest {
    pub fn unpack<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        serde_json::from_slice(self.content.as_slice()).ok()
    }

    pub fn reply<T: Serialize>(self, value: &T) -> RpcResult<()> {
        let content = serde_json::to_vec(value).unwrap();
        self.resp
            .send(content)
            .map_err(|_| RpcError::ConnectionRefused)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn rpc_impl<T: Serialize>(
    from: Address,
    to: SocketAddr,
    tag: u64,
    value: T,
) -> Result<RpcResponse, RpcError> {
    let addr = format!("http://{}:{}", to.ip(), to.port());
    let mut client = RpcClient::connect(addr)
        .await
        .map_err(|_| RpcError::ConnectionRefused)?;
    let content = serde_json::to_vec(&value).unwrap();
    let request = real::rpc::server::real_rpc::RpcRequest {
        from: from.to_string(),
        tag,
        content,
    };
    let request = tonic::Request::new(request);
    let result = client
        .send(request)
        .await
        .map_err(|_| RpcError::ConnectionRefused)?;
    let response = result.into_inner();
    let response = RpcResponse {
        content: response.content,
    };
    Ok(response)
}

////////////////////////////////////////////////////////////////////////////////

pub async fn rpc<T: Serialize>(to: Address, tag: u64, value: &T) -> RpcResult<RpcResponse> {
    Context::current().rpc(to, tag, value).await
}
