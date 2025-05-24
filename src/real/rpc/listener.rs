use tokio::sync::mpsc::UnboundedReceiver;

use super::RpcRequest;

////////////////////////////////////////////////////////////////////////////////

pub struct RpcListener {
    pub(crate) receiver: UnboundedReceiver<RpcRequest>,
}

impl RpcListener {
    pub async fn listen(&mut self) -> RpcRequest {
        self.receiver.recv().await.unwrap()
    }
}
