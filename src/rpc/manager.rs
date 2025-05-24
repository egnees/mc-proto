use std::collections::{hash_map::Entry, HashMap};

use crate::{
    util::{
        oneshot,
        unbounded::{make_channel, Sender},
    },
    Address, RpcError, RpcResult,
};

use super::{listener::RpcListener, request::RpcRequest, RpcResponse};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct RpcManager {
    next_id: u64,
    listeners: HashMap<Address, Sender<RpcRequest>>,
    req: HashMap<u64, oneshot::Sender<RpcResult<RpcResponse>>>,
}

impl RpcManager {
    pub fn inc_next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn has_listener(&self, addr: &Address) -> bool {
        self.listeners
            .get(addr)
            .map(|s| s.receiver_alive())
            .unwrap_or(false)
    }

    pub fn register_listener(&mut self, from: Address) -> RpcResult<RpcListener> {
        let entry = self.listeners.entry(from);
        match entry {
            Entry::Occupied(mut e) => {
                if e.get().receiver_alive() {
                    Err(RpcError::AlreadyListening)
                } else {
                    let (rx, tx) = make_channel::<RpcRequest>();
                    let listener = RpcListener::new(tx);
                    e.insert(rx);
                    Ok(listener)
                }
            }
            Entry::Vacant(e) => {
                let (rx, tx) = make_channel::<RpcRequest>();
                let listener = RpcListener::new(tx);
                e.insert(rx);
                Ok(listener)
            }
        }
    }

    pub fn send_request(
        &mut self,
        mut request: RpcRequest,
    ) -> RpcResult<oneshot::Receiver<RpcResult<RpcResponse>>> {
        let id = request.id;
        let sender = self
            .listeners
            .get(&request.to)
            .ok_or(RpcError::ConnectionRefused)?;
        request.await_response();
        let result = sender.send(request);
        if let Err(mut e) = result {
            e.not_await_response();
            Err(RpcError::ConnectionRefused)
        } else {
            let (sender, receiver) = oneshot::channel();
            let prev = self.req.insert(id, sender);
            assert!(prev.is_none());
            Ok(receiver)
        }
    }

    pub fn send_response(
        &mut self,
        request_id: u64,
        response: RpcResult<RpcResponse>,
    ) -> RpcResult<()> {
        let sender = self.req.remove(&request_id).unwrap();
        sender
            .send(response)
            .map_err(|_| RpcError::ConnectionRefused)
    }

    pub fn response_receiver_alive(&self, request_id: u64) -> bool {
        self.req
            .get(&request_id)
            .map(|s| s.has_receiver())
            .unwrap_or(false)
    }
}
