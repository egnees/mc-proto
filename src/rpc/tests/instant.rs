use std::collections::HashMap;

use crate::rpc::manager::RpcManager;
use crate::rpc::registry::RpcRegistry;

use crate::{util::oneshot, Address};

use crate::rpc::{
    error::RpcResult, listener::RpcListener, request::RpcRequest, response::RpcResponse,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct InstantRpcRegistry {
    next_id: u64,
    manager: RpcManager,
    req: HashMap<u64, oneshot::Sender<RpcResult<RpcResponse>>>,
}

impl RpcRegistry for InstantRpcRegistry {
    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn register_request(
        &mut self,
        request: RpcRequest,
    ) -> oneshot::Receiver<RpcResult<RpcResponse>> {
        let (tx, rx) = oneshot::channel::<RpcResult<RpcResponse>>();
        let id = request.id;
        let result = self.manager.send_request(request);
        if result.is_err() {
            tx.send(Err(result.unwrap_err())).unwrap();
        } else {
            let prev = self.req.insert(id, tx);
            assert!(prev.is_none());
        }
        rx
    }

    fn register_response(
        &mut self,
        _from: Address,
        _to: Address,
        request_id: u64,
        response: RpcResponse,
    ) -> RpcResult<()> {
        let sender = self.req.remove(&request_id).unwrap();
        let _ = sender.send(Ok(response));
        Ok(())
    }

    fn register_listener(&mut self, from: Address) -> RpcResult<RpcListener> {
        self.manager.register_listener(from)
    }
}
