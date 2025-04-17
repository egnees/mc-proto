use std::{cell::RefCell, rc::Rc};

use crate::{
    sim::context::Context,
    util::{self, trigger::make_trigger},
    Address,
};

use super::{error::TcpError, packet::TcpPacket, registry::TcpRegistry};

////////////////////////////////////////////////////////////////////////////////

pub struct TcpStream {
    registry: Rc<RefCell<dyn TcpRegistry>>,
    pub(crate) from: Address,
    pub(crate) to: Address,
    pub(crate) sender: Rc<util::append::Sender>,
    receiver: util::append::Receiver,
    connected: bool,
}

////////////////////////////////////////////////////////////////////////////////

async fn send_and_wait_delivery(
    from: Address,
    to: Address,
    packet: TcpPacket,
    registry: Rc<RefCell<dyn TcpRegistry>>,
) -> Result<(), TcpError> {
    let (waiter, trigger) = make_trigger();

    registry
        .borrow_mut()
        .emit_packet(&from, &to, &packet, trigger)?;

    waiter.wait::<Result<(), TcpError>>().await.unwrap()?;

    registry
        .borrow_mut()
        .register_packet_delivery(&from, &to, &packet)
}

////////////////////////////////////////////////////////////////////////////////

impl TcpStream {
    pub(crate) fn mark_connected(&mut self) {
        self.connected = true;
    }

    pub(crate) fn unmark_connected(&mut self) {
        self.connected = false;
    }

    /// Emit packet from this to opposite and wait for packet delivery.
    async fn send_and_wait_delivery_dirrect(&self, packet: TcpPacket) -> Result<(), TcpError> {
        send_and_wait_delivery(
            self.from.clone(),
            self.to.clone(),
            packet,
            self.registry.clone(),
        )
        .await
    }

    /// Emit packet from opposite to this and wait delivery
    /// Should not return error.
    async fn send_and_wait_delivery_opposite(&self, packet: TcpPacket) -> Result<(), TcpError> {
        send_and_wait_delivery(
            self.to.clone(),
            self.from.clone(),
            packet,
            self.registry.clone(),
        )
        .await
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn send(&mut self, bytes: &[u8]) -> Result<usize, TcpError> {
        let packet = TcpPacket::Data(bytes.to_vec());
        let send_result = self
            .send_and_wait_delivery_dirrect(packet)
            .await
            .map(|_| bytes.len());
        if send_result.is_ok() {
            let sent = self.sender.send(bytes);
            assert!(sent);
            self.send_and_wait_delivery_opposite(TcpPacket::Ack())
                .await?;
        } else {
            self.send_and_wait_delivery_opposite(TcpPacket::Nack())
                .await?;
        }
        send_result
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn recv(&mut self, bytes: &mut [u8]) -> Result<usize, TcpError> {
        self.receiver
            .recv(bytes)
            .await
            .ok_or(TcpError::ConnectionRefused) // must fail on sender drop
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) async fn connect_addr(
        from: Address,
        to: Address,
        registry: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        // send connect request
        send_and_wait_delivery(
            from.clone(),
            to.clone(),
            TcpPacket::Connect(),
            registry.clone(),
        )
        .await?;

        // here `to` must listen
        let conn = registry
            .borrow_mut()
            .try_connect(&from, &to, registry.clone())?;

        // send ack
        send_and_wait_delivery(to, from, TcpPacket::Ack(), registry).await?;

        Ok(conn)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) async fn listen(
        from: Address,
        registry: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        let (waiter, trigger) = make_trigger();
        registry.borrow_mut().emit_listen_request(&from, trigger)?;
        waiter.wait::<Result<TcpStream, TcpError>>().await.unwrap()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) async fn listen_to(
        from: Address,
        to: Address,
        registry: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        let (waiter, trigger) = make_trigger();
        registry
            .borrow_mut()
            .emit_listen_to_request(&from, &to, trigger)?;
        waiter.wait::<Result<TcpStream, TcpError>>().await.unwrap()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn connect(to: &Address) -> Result<Self, TcpError> {
        let ctx = Context::current();
        let registry = ctx.event_manager.tcp_registry();
        let from = Context::current().proc.address();
        Self::connect_addr(from, to.clone(), registry).await
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Drop for TcpStream {
    fn drop(&mut self) {
        if self.connected {
            self.registry.clone().borrow_mut().emit_disconnect(self);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn make_connection(
    a: Address,
    b: Address,
    registry: Rc<RefCell<dyn TcpRegistry>>,
) -> (TcpStream, TcpStream) {
    let (s1, r1) = util::append::mpsc_channel();
    let (s2, r2) = util::append::mpsc_channel();
    let s1 = Rc::new(s1);
    let s2 = Rc::new(s2);
    let first = TcpStream {
        registry: registry.clone(),
        from: a.clone(),
        to: b.clone(),
        sender: s1,
        receiver: r2,
        connected: false,
    };
    let second = TcpStream {
        registry,
        from: b.clone(),
        to: a.clone(),
        sender: s2,
        receiver: r1,
        connected: false,
    };
    (first, second)
}
