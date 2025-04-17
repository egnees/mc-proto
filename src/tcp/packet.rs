use std::{fmt::Display, hash::Hash};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum TcpPacket {
    Connect(),
    Disconnect(),
    Data(Vec<u8>),
    Ack(),
    Nack(),
}

////////////////////////////////////////////////////////////////////////////////

impl Display for TcpPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TcpPacket::Connect() => write!(f, "CONNECT"),
            TcpPacket::Disconnect() => write!(f, "DISCONNECT"),
            TcpPacket::Data(data) => match String::from_utf8(data.clone()) {
                Ok(s) => write!(f, "{}", s),
                Err(_) => write!(f, "{:?}", data),
            },
            TcpPacket::Ack() => write!(f, "ACK"),
            TcpPacket::Nack() => write!(f, "NACK"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Hash for TcpPacket {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TcpPacket::Connect() => 0.hash(state),
            TcpPacket::Disconnect() => 1.hash(state),
            TcpPacket::Data(data) => {
                2.hash(state);
                data.hash(state)
            }
            TcpPacket::Ack() => 3.hash(state),
            TcpPacket::Nack() => 4.hash(state),
        };
    }
}
