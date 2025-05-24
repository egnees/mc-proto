use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum Error {
    #[error("already exists")]
    AlreadyExists,
    #[error("not found")]
    NotFound,
    #[error("socket address for process is not registered")]
    SocketNotRegistered,
}
