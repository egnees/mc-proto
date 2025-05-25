use std::fmt::Display;

////////////////////////////////////////////////////////////////////////////////

/// Represents TCP error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TcpError {
    /// The connection refused
    ConnectionRefused,

    /// Trying to listen,
    /// but process is already listening to the TCP connections.
    AlreadyListening,
}

////////////////////////////////////////////////////////////////////////////////

impl Display for TcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TcpError::ConnectionRefused => write!(f, "connection refused"),
            TcpError::AlreadyListening => write!(f, "already listening on this address"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> From<TcpError> for Result<T, TcpError> {
    fn from(value: TcpError) -> Self {
        Err(value)
    }
}
