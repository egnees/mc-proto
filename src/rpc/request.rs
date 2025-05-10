use std::{cell::RefCell, rc::Rc};

use serde::Serialize;

use crate::Address;

use super::{error::RpcResult, registry::RpcRegistry, response::RpcResponse};

////////////////////////////////////////////////////////////////////////////////

pub struct RpcRequest {
    pub(crate) id: u64,
    reg: Rc<RefCell<dyn RpcRegistry>>,
    pub(crate) from: Address,
    pub(crate) to: Address,
    pub tag: u64,
    pub content: Vec<u8>,
}

impl RpcRequest {
    pub fn new(
        reg: Rc<RefCell<dyn RpcRegistry>>,
        id: u64,
        from: Address,
        to: Address,
        tag: u64,
        content: Vec<u8>,
    ) -> Self {
        Self {
            reg,
            id,
            from,
            to,
            tag,
            content,
        }
    }

    pub fn reply<T: Serialize>(self, value: &T) -> RpcResult<()> {
        let reply = RpcResponse::new_with_type(value)?;
        self.reply_impl(reply)
    }

    fn reply_impl(self, r: RpcResponse) -> RpcResult<()> {
        self.reg
            .borrow_mut()
            .register_response(self.to, self.from, self.id, r)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub async fn rpc<T: Serialize>(
    from: Address,
    to: Address,
    reg: Rc<RefCell<dyn RpcRegistry>>,
    tag: u64,
    value: &T,
) -> RpcResult<RpcResponse> {
    let id = reg.borrow_mut().next_request_id();
    let content = serde_json::to_vec(value)?;
    let request = RpcRequest {
        id,
        reg: reg.clone(),
        from,
        to,
        tag,
        content,
    };
    let receiver = reg.borrow_mut().register_request(request);
    receiver.await.unwrap()
}
