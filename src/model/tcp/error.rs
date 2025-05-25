use std::fmt::Display;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TcpError {
    ConnectionRefused,
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
