use std::{cell::RefCell, rc::Rc};

use crate::{
    log,
    sim::context::Context,
    spawn,
    util::{self, trigger::make_trigger},
    Address,
};

use super::{
    error::TcpError,
    packet::{TcpPacket, TcpPacketKind},
    registry::TcpRegistry,
};

////////////////////////////////////////////////////////////////////////////////

pub struct TcpStream {
    pub(crate) sender: TcpSender,
    pub(crate) receiver: TcpReceiver,
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

pub struct TcpSender {
    pub(crate) stream_id: usize,
    registry: Rc<RefCell<dyn TcpRegistry>>,
    pub(crate) me: Address,
    pub(crate) other: Address,
    pub(crate) sender: Rc<util::append::Sender>,
    connected: bool,
}

impl TcpSender {
    fn new(
        stream_id: usize,
        registry: Rc<RefCell<dyn TcpRegistry>>,
        me: Address,
        other: Address,
        sender: Rc<util::append::Sender>,
    ) -> Self {
        Self {
            stream_id,
            registry,
            me,
            other,
            sender,
            connected: false,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn mark_connected(&mut self) {
        self.connected = true;
    }

    pub(crate) fn unmark_connected(&mut self) {
        self.connected = false;
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn packet(&self, kind: TcpPacketKind) -> TcpPacket {
        TcpPacket::new(self.stream_id, kind)
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Emit packet from this to opposite and wait for packet delivery.
    async fn send_and_wait_delivery_dirrect(&self, kind: TcpPacketKind) -> Result<(), TcpError> {
        let packet = self.packet(kind);
        send_and_wait_delivery(
            self.me.clone(),
            self.other.clone(),
            packet,
            self.registry.clone(),
        )
        .await
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Emit packet from opposite to this and wait delivery
    /// Should not return error.
    async fn send_and_wait_delivery_opposite(&self, kind: TcpPacketKind) -> Result<(), TcpError> {
        let packet = self.packet(kind);
        send_and_wait_delivery(
            self.other.clone(),
            self.me.clone(),
            packet,
            self.registry.clone(),
        )
        .await
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn send(&self, bytes: &[u8]) -> Result<usize, TcpError> {
        let packet = TcpPacketKind::Data(bytes.to_vec());
        let send_result = self
            .send_and_wait_delivery_dirrect(packet)
            .await
            .map(|_| bytes.len())?;
        let sent = self.sender.send(bytes);
        assert!(sent);
        self.send_and_wait_delivery_opposite(TcpPacketKind::Ack())
            .await?;
        Ok(send_result)
    }

    pub fn send_sync(&self, bytes: &[u8]) -> Result<usize, TcpError> {
        let packet = self.packet(TcpPacketKind::Data(bytes.to_vec()));
        let from = self.me.clone();
        let to = self.other.clone();
        let reg = self.registry.clone();
        spawn(async {
            let _ = send_and_wait_delivery(from, to, packet, reg).await;
        });
        Ok(bytes.len())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct TcpReceiver {
    receiver: util::append::Receiver,
}

impl TcpReceiver {
    fn new(receiver: util::append::Receiver) -> Self {
        Self { receiver }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn recv(&mut self, bytes: &mut [u8]) -> Result<usize, TcpError> {
        self.receiver
            .recv(bytes)
            .await
            .ok_or(TcpError::ConnectionRefused) // must fail on sender drop
    }
}

////////////////////////////////////////////////////////////////////////////////

impl TcpStream {
    fn new(
        stream_id: usize,
        reg: Rc<RefCell<dyn TcpRegistry>>,
        from: Address,
        to: Address,
        sender: util::append::Sender,
        receiver: util::append::Receiver,
    ) -> Self {
        let sender = TcpSender::new(stream_id, reg, from, to, Rc::new(sender));
        let receiver = TcpReceiver::new(receiver);
        Self { sender, receiver }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn from(&self) -> &Address {
        &self.sender.me
    }

    pub fn to(&self) -> &Address {
        &self.sender.other
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) fn mark_connected(&mut self) {
        self.sender.mark_connected();
    }

    pub(crate) fn unmark_connected(&mut self) {
        self.sender.unmark_connected();
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn send(&self, bytes: &[u8]) -> Result<usize, TcpError> {
        self.sender.send(bytes).await
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn send_sync(&self, bytes: &[u8]) -> Result<usize, TcpError> {
        self.sender.send_sync(bytes)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn recv(&mut self, bytes: &mut [u8]) -> Result<usize, TcpError> {
        self.receiver.recv(bytes).await
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub(crate) async fn connect_addr(
        from: Address,
        to: Address,
        registry: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        let stream_id = registry.borrow_mut().next_tcp_stream_id();

        // send connect request
        send_and_wait_delivery(
            from.clone(),
            to.clone(),
            TcpPacket::new(stream_id, TcpPacketKind::Connect()),
            registry.clone(),
        )
        .await?;

        // here `to` must listen
        let conn = registry
            .borrow_mut()
            .try_connect(&from, &to, stream_id, registry.clone())?;

        // send ack
        send_and_wait_delivery(
            to,
            from,
            TcpPacket::new(stream_id, TcpPacketKind::Ack()),
            registry,
        )
        .await?;

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

    ////////////////////////////////////////////////////////////////////////////////

    pub fn split(self) -> (TcpSender, TcpReceiver) {
        (self.sender, self.receiver)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Drop for TcpSender {
    fn drop(&mut self) {
        if self.connected {
            log("tcp sender drop");
            self.registry.clone().borrow_mut().emit_sender_dropped(self);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn make_connection(
    a: Address,
    b: Address,
    stream_id: usize,
    reg: Rc<RefCell<dyn TcpRegistry>>,
) -> (TcpStream, TcpStream) {
    let (s1, r1) = util::append::mpsc_channel();
    let (s2, r2) = util::append::mpsc_channel();
    let first = TcpStream::new(stream_id, reg.clone(), a.clone(), b.clone(), s1, r2);
    let second = TcpStream::new(stream_id, reg, b, a, s2, r1);
    (first, second)
}
