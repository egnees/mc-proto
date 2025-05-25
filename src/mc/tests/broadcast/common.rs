use crate::{model, Address};

use super::causal::Mailman;

////////////////////////////////////////////////////////////////////////////////

pub trait BuildFn: Fn(model::SystemHandle, usize) + Send + Sync + Clone + 'static {}

impl<F: Fn(model::SystemHandle, usize) + Send + Sync + Clone + 'static> BuildFn for F {}

////////////////////////////////////////////////////////////////////////////////

pub fn send_local(s: model::SystemHandle, node: usize, msg: impl Into<String>) -> bool {
    let address = format!("{node}:bcast").into();
    s.send_local(&address, msg).is_ok()
}

////////////////////////////////////////////////////////////////////////////////

pub fn read_locals(s: model::SystemHandle, node: usize) -> Result<Vec<String>, String> {
    let address: Address = format!("{node}:bcast").into();
    s.read_locals(address.node, address.process)
        .ok_or("No such address".into())
}

////////////////////////////////////////////////////////////////////////////////

pub struct LocalMail {}

impl Mailman for LocalMail {
    fn deliver(&mut self, msg: &str) {
        crate::send_local(msg);
    }
}
