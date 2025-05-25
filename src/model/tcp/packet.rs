use std::{fmt::Display, hash::Hash};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TcpPacketKind {
    Connect(),
    Data(Vec<u8>),
    Ack(),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TcpPacket {
    pub tcp_stream_id: usize,
    pub kind: TcpPacketKind,
}

impl TcpPacket {
    pub fn new(stream: usize, kind: TcpPacketKind) -> Self {
        Self {
            tcp_stream_id: stream,
            kind,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Display for TcpPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TcpPacketKind::Connect() => write!(f, "CONNECT"),
            TcpPacketKind::Data(data) => match String::from_utf8(data.clone()) {
                Ok(s) => write!(f, "{}", s),
                Err(_) => write!(f, "{:?}", data),
            },
            TcpPacketKind::Ack() => write!(f, "ACK"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Hash for TcpPacket {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.kind {
            TcpPacketKind::Connect() => 0.hash(state),
            TcpPacketKind::Data(data) => {
                1.hash(state);
                data.hash(state)
            }
            TcpPacketKind::Ack() => 2.hash(state),
        };
    }
}
