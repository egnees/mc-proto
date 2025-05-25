use crate::common::RpcResult;
use crate::{model::context::Context, util::unbounded::Receiver};

use super::request::RpcRequest;

////////////////////////////////////////////////////////////////////////////////

pub struct RpcListener {
    queue: Receiver<RpcRequest>,
}

impl RpcListener {
    pub(crate) fn new(r: Receiver<RpcRequest>) -> Self {
        Self { queue: r }
    }

    pub async fn listen(&mut self) -> RpcRequest {
        self.queue.recv().await.unwrap()
    }

    pub fn register() -> RpcResult<Self> {
        let ctx = Context::current();
        let rpc = ctx.event_manager.rpc_registry();
        let from = ctx.proc.address();
        let result = rpc.borrow_mut().register_listener(from);
        result
    }
}
