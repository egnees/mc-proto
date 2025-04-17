use std::{cell::RefCell, rc::Rc};

use crate::{util::trigger::Trigger, Address};

use super::{error::TcpError, packet::TcpPacket, stream::TcpStream};

////////////////////////////////////////////////////////////////////////////////

pub trait TcpRegistry {
    fn emit_packet(
        &mut self,
        from: &Address,
        to: &Address,
        packet: &TcpPacket,
        on_delivery: Trigger,
    ) -> Result<(), TcpError>;

    fn emit_listen_request(&mut self, from: &Address, on_listen: Trigger) -> Result<(), TcpError>;

    fn emit_listen_to_request(
        &mut self,
        from: &Address,
        to: &Address,
        on_listen: Trigger,
    ) -> Result<(), TcpError>;

    fn emit_disconnect(&mut self, sender: &mut TcpStream);

    ////////////////////////////////////////////////////////////////////////////////

    fn register_packet_delivery(
        &mut self,
        from: &Address,
        to: &Address,
        packet: &TcpPacket,
    ) -> Result<(), TcpError>;

    fn try_connect(
        &mut self,
        from: &Address,
        to: &Address,
        registry_ref: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError>;
}
