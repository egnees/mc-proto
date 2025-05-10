use crate::{util::oneshot, Address};

use super::{error::RpcResult, listener::RpcListener, request::RpcRequest, response::RpcResponse};

////////////////////////////////////////////////////////////////////////////////

pub trait RpcRegistry {
    fn next_request_id(&mut self) -> u64;

    fn register_request(
        &mut self,
        request: RpcRequest,
    ) -> oneshot::Receiver<RpcResult<RpcResponse>>;

    fn register_response(
        &mut self,
        from: Address,
        to: Address,
        request_id: u64,
        response: RpcResult<RpcResponse>,
    ) -> RpcResult<()>;

    fn register_listener(&mut self, from: Address) -> RpcResult<RpcListener>;
}
