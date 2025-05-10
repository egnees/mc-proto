use crate::util::unbounded::Receiver;

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
}
