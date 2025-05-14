use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    #[error("file {file:?} not found")]
    FileNotFound { file: String },
    #[error("file {file:?} already exists")]
    FileAlreadyExists { file: String },
    #[error("reached limit of storage capacity")]
    StorageLimitReached,
    #[error("storage is not available")]
    StorageNotAvailable,
    #[error("file not available")]
    FileNotAvailable,
}

////////////////////////////////////////////////////////////////////////////////

pub type FsResult<T> = Result<T, FsError>;
