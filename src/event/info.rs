use std::{fmt::Display, hash::Hash, time::Duration};

use crate::{
    fs::event::{FsEventKind, FsEventOutcome},
    sim::proc::ProcessHandle,
    tcp::packet::TcpPacket,
    Address, RpcError, RpcResult, TcpError,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Hash)]
pub enum EventInfo {
    UdpMessage(UdpMessage),
    TcpMessage(TcpMessage),
    TcpEvent(TcpEvent),
    RpcMessage(RpcMessage),
    RpcEvent(RpcEvent),
    FsEvent(FsEvent),
    Timer(Timer),
}

impl Display for EventInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventInfo::UdpMessage(udp_message) => write!(f, "Udp Message: {{{}}}", udp_message),
            EventInfo::TcpMessage(tcp_message) => write!(f, "Tcp message: {{{}}}", tcp_message),
            EventInfo::Timer(timer) => write!(f, "Timer: {}", timer),
            EventInfo::TcpEvent(tcp_event) => write!(f, "Tcp event: {{{}}}", tcp_event),
            EventInfo::FsEvent(fs_event) => write!(f, "Fs event: {{{}}}", fs_event),
            EventInfo::RpcMessage(rpc_msg) => write!(f, "Rpc message: {{{}}}", rpc_msg),
            EventInfo::RpcEvent(rpc_event) => write!(f, "Rpc event: {{{}}}", rpc_event),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum RpcMessageKind {
    Request {
        id: u64,
        tag: u64,
        content: Vec<u8>,
    },
    Response {
        id: u64,
        content: RpcResult<Vec<u8>>,
    },
}

impl Hash for RpcMessageKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RpcMessageKind::Request { tag, content, .. } => {
                tag.hash(state);
                content.hash(state);
            }
            RpcMessageKind::Response { content, .. } => {
                content.hash(state);
            }
        }
    }
}

impl Display for RpcMessageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcMessageKind::Request { id, tag, content } => write!(
                f,
                "request, id: {}, tag: {}, content: {:?}",
                id,
                tag,
                String::from_utf8(content.clone()).unwrap()
            ),
            RpcMessageKind::Response { id, content } => {
                write!(
                    f,
                    "response, id: {}, content: {:?}",
                    id,
                    match content {
                        Ok(c) => {
                            String::from_utf8(c.clone()).unwrap()
                        }
                        Err(e) => {
                            e.to_string()
                        }
                    }
                )
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RpcMessage {
    pub from: ProcessHandle,
    pub to: ProcessHandle,
    pub kind: RpcMessageKind,
}

impl Hash for RpcMessage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.address().hash(state);
        self.to.address().hash(state);
        self.kind.hash(state);
    }
}

impl Display for RpcMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "from: '{}', ", self.from.address())?;
        write!(f, "to: '{}', ", self.to.address())?;
        write!(f, "kind: '{}'", self.kind)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Hash)]
pub enum RpcEventKind {
    ConnectionRefused,
}

impl RpcEventKind {
    pub fn rpc_result(&self) -> RpcResult<()> {
        match self {
            RpcEventKind::ConnectionRefused => Err(RpcError::ConnectionRefused),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RpcEvent {
    pub kind: RpcEventKind,
    pub to: ProcessHandle,
}

impl Hash for RpcEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.to.address().hash(state);
    }
}

impl Display for RpcEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rpc event to: '{}'", self.to.address())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum TcpEventKind {
    SenderDropped,
    #[allow(unused)]
    ReceiverDropped,
    ConnectionRefused,
}

impl TcpEventKind {
    pub fn tcp_result(&self) -> Result<(), TcpError> {
        match self {
            TcpEventKind::SenderDropped => Err(TcpError::ConnectionRefused),
            TcpEventKind::ReceiverDropped => Err(TcpError::ConnectionRefused),
            TcpEventKind::ConnectionRefused => Err(TcpError::ConnectionRefused),
        }
    }
}

impl Hash for TcpEventKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TcpEventKind::SenderDropped => 0.hash(state),
            TcpEventKind::ReceiverDropped => 1.hash(state),
            TcpEventKind::ConnectionRefused => 2.hash(state),
        }
    }
}

impl Display for TcpEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TcpEventKind::SenderDropped => write!(f, "sender dropped"),
            TcpEventKind::ReceiverDropped => write!(f, "receiver dropped"),
            TcpEventKind::ConnectionRefused => write!(f, "connection refused"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TcpEvent {
    pub kind: TcpEventKind,
    pub to: ProcessHandle,
}

impl Hash for TcpEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.to.address().hash(state);
    }
}

impl Display for TcpEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tcp event to '{}'", self.to.address())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TcpMessage {
    pub tcp_msg_id: usize,
    pub from: ProcessHandle,
    pub to: ProcessHandle,
    pub packet: TcpPacket,
}

impl Hash for TcpMessage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.address().hash(state);
        self.to.address().hash(state);
        self.packet.hash(state);
    }
}

impl Display for TcpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, ", self.tcp_msg_id)?;
        write!(f, "from: '{}', ", self.from.address())?;
        write!(f, "to: '{}', ", self.to.address())?;
        write!(f, "packet: {}", self.packet)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct UdpMessage {
    pub udp_msg_id: usize,
    pub from: ProcessHandle,
    pub to: ProcessHandle,
    pub content: String,
}

impl Hash for UdpMessage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.address().hash(state);
        self.to.address().hash(state);
        self.content.hash(state);
    }
}

impl Display for UdpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {},  ", self.udp_msg_id)?;
        write!(f, "from: '{}', ", self.from.address())?;
        write!(f, "to: '{}', ", self.to.address())?;
        write!(f, "content: '{}'", self.content)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Timer {
    pub timer_id: usize,
    pub with_sleep: bool,
    pub proc: ProcessHandle,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

impl Hash for Timer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.min_duration.hash(state);
        self.max_duration.hash(state);
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, ", self.timer_id)?;
        write!(f, "proc: '{}', ", self.proc.address())?;
        write!(f, "min_duration: {:?}", self.min_duration)?;
        write!(f, "max_duration: {:?}", self.max_duration)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct FsEvent {
    pub proc: Address,
    pub kind: FsEventKind,
    pub outcome: FsEventOutcome,
}

impl Hash for FsEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Display for FsEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}
