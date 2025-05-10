use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone, Hash)]
pub enum RpcError {
    #[error("internal: {info}")]
    Internal { info: String },
    #[error("already listening for rpc requests")]
    AlreadyListening,
    #[error("connection refused")]
    ConnectionRefused,
}

impl From<serde_json::Error> for RpcError {
    fn from(value: serde_json::Error) -> Self {
        Self::Internal {
            info: value.to_string(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub type RpcResult<T> = Result<T, RpcError>;
