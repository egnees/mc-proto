use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{sim::context::Context, Address};

use super::{error::RpcResult, registry::RpcRegistry, response::RpcResponse, RpcError};

////////////////////////////////////////////////////////////////////////////////

pub struct RpcRequest {
    pub(crate) id: u64,
    reg: Rc<RefCell<dyn RpcRegistry>>,
    pub(crate) from: Address,
    pub(crate) to: Address,
    pub tag: u64,
    pub content: Vec<u8>,
    await_response: bool,
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
            await_response: false,
        }
    }

    pub fn unpack<'a, T: Deserialize<'a>>(&'a self) -> RpcResult<T> {
        Ok(serde_json::from_slice(&self.content)?)
    }

    pub fn reply<T: Serialize>(self, value: &T) -> RpcResult<()> {
        let reply = RpcResponse::new_with_type(self.id, value)?;
        self.reply_impl(reply)
    }

    fn reply_impl(mut self, r: RpcResponse) -> RpcResult<()> {
        self.not_await_response();
        self.reg
            .borrow_mut()
            .register_response(self.to.clone(), self.from.clone(), self.id, Ok(r))
    }

    pub(crate) fn await_response(&mut self) {
        self.await_response = true;
    }

    pub(crate) fn not_await_response(&mut self) {
        self.await_response = false;
    }

    pub fn from(&self) -> &Address {
        &self.from
    }
}

impl Drop for RpcRequest {
    fn drop(&mut self) {
        if self.await_response {
            let _ = self.reg.borrow_mut().register_response(
                self.to.clone(),
                self.from.clone(),
                self.id,
                Err(RpcError::ConnectionRefused),
            );
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub async fn rpc_impl<T: Serialize>(
    from: Address,
    to: Address,
    reg: Rc<RefCell<dyn RpcRegistry>>,
    tag: u64,
    value: &T,
) -> RpcResult<RpcResponse> {
    let id = reg.borrow_mut().next_request_id();
    let content = serde_json::to_vec(value)?;
    let request = RpcRequest::new(reg.clone(), id, from, to, tag, content);
    let receiver = reg.borrow_mut().register_request(request);
    receiver.await.unwrap()
}

////////////////////////////////////////////////////////////////////////////////

pub async fn rpc<T: Serialize>(to: Address, tag: u64, value: &T) -> RpcResult<RpcResponse> {
    let ctx = Context::current();
    let reg = ctx.event_manager.rpc_registry();
    let from = ctx.proc.address();
    rpc_impl(from, to, reg, tag, value).await
}
