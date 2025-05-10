use std::collections::{hash_map::Entry, HashMap};

use crate::{
    util::unbounded::{make_channel, Sender},
    Address,
};

use super::{
    error::{RpcError, RpcResult},
    listener::RpcListener,
    request::RpcRequest,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct RpcManager {
    next_id: u64,
    listeners: HashMap<Address, Sender<RpcRequest>>,
}

impl RpcManager {
    pub fn inc_next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
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

    pub fn send_request(&mut self, request: RpcRequest) -> RpcResult<()> {
        let result = self
            .listeners
            .get(&request.to)
            .ok_or(RpcError::ConnectionRefused)?
            .send(request);
        if result.is_err() {
            Err(RpcError::ConnectionRefused)
        } else {
            Ok(())
        }
    }
}
