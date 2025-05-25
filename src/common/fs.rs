use thiserror::Error;

use crate::model;
use crate::real;

use super::mode::is_real;

////////////////////////////////////////////////////////////////////////////////

/// Represents file system error.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    /// Requested file not found.
    #[error("file {file:?} not found")]
    FileNotFound {
        /// File name
        file: String,
    },

    /// Trying to create already existent file.
    #[error("file {file:?} already exists")]
    FileAlreadyExists {
        /// File name
        file: String,
    },

    /// Reached limit of storage capacity.
    #[error("reached limit of storage capacity")]
    StorageLimitReached,

    /// The storage is not available now.
    #[error("storage is not available")]
    StorageNotAvailable,

    /// The file is not available now.
    #[error("file not available")]
    FileNotAvailable,

    /// Failed to resolve path.
    #[error("bad path")]
    BadPath {
        /// Path
        path: String,
    },
}

////////////////////////////////////////////////////////////////////////////////

/// Represents type of file system error.
pub type FsResult<T> = Result<T, FsError>;

////////////////////////////////////////////////////////////////////////////////

/// Represents file.
pub enum File {
    /// Real file on the real file system
    Real(real::File),

    /// Model of file
    Model(model::File),
}

impl From<real::File> for File {
    fn from(value: real::File) -> Self {
        Self::Real(value)
    }
}

impl From<model::File> for File {
    fn from(value: model::File) -> Self {
        Self::Model(value)
    }
}

impl File {
    /// Allows to clone file
    pub async fn try_clone(&self) -> FsResult<Self> {
        match self {
            File::Real(file) => file.try_clone().await.map(File::from),
            File::Model(file) => Ok(file.clone().into()),
        }
    }

    /// Read bytes from file
    pub async fn read(&mut self, buf: &mut [u8], offset: usize) -> FsResult<usize> {
        match self {
            File::Real(file) => file.read(buf, offset).await,
            File::Model(file) => file.read(buf, offset).await,
        }
    }

    /// Write bytes to file
    pub async fn write(&mut self, buf: &[u8], offset: usize) -> FsResult<usize> {
        match self {
            File::Real(file) => file.write(buf, offset).await,
            File::Model(file) => file.write(buf, offset).await,
        }
    }

    /// Open file
    pub async fn open(name: impl Into<String>) -> FsResult<Self> {
        if is_real() {
            real::File::open(name).await.map(File::from)
        } else {
            model::File::open(name).map(File::from)
        }
    }

    /// Create file
    pub async fn create(name: impl Into<String>) -> FsResult<Self> {
        if is_real() {
            real::File::create(name).await.map(File::from)
        } else {
            model::File::create(name).map(File::from)
        }
    }

    /// Delete file
    pub async fn delete(name: impl Into<String>) -> FsResult<()> {
        if is_real() {
            real::File::delete(name).await
        } else {
            model::File::delete(name)
        }
    }
}
