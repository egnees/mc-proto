use crate::rpc::manager::RpcManager;
use crate::rpc::registry::RpcRegistry;

use crate::{util::oneshot, Address, RpcResult};

use crate::rpc::{listener::RpcListener, request::RpcRequest, response::RpcResponse};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct InstantRpcRegistry {
    manager: RpcManager,
}

impl RpcRegistry for InstantRpcRegistry {
    fn next_request_id(&mut self) -> u64 {
        self.manager.inc_next_id()
    }

    fn register_request(
        &mut self,
        request: RpcRequest,
    ) -> oneshot::Receiver<RpcResult<RpcResponse>> {
        let result = self.manager.send_request(request);
        match result {
            Ok(recv) => recv,
            Err(e) => {
                let (tx, rx) = oneshot::channel();
                tx.send(Err(e)).unwrap();
                rx
            }
        }
    }

    fn register_response(
        &mut self,
        _from: Address,
        _to: Address,
        request_id: u64,
        response: RpcResult<RpcResponse>,
    ) -> RpcResult<()> {
        self.manager.send_response(request_id, response)
    }

    fn register_listener(&mut self, from: Address) -> RpcResult<RpcListener> {
        self.manager.register_listener(from)
    }
}
