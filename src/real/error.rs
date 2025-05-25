use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

/// Represents error which can happen in real mode.
#[derive(Error, Debug)]
pub enum Error {
    /// Resource already exists
    #[error("already exists")]
    AlreadyExists,

    #[error("not found")]
    /// Resource not found.
    NotFound,

    /// Socket for the address is not registered.
    #[error("socket address for process is not registered")]
    SocketNotRegistered,
}
