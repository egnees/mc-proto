use thiserror::Error;

use crate::model;
use crate::real;

use super::mode::is_real;

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
    #[error("bad path")]
    BadPath { path: String },
}

////////////////////////////////////////////////////////////////////////////////

pub type FsResult<T> = Result<T, FsError>;

////////////////////////////////////////////////////////////////////////////////

pub enum File {
    Real(real::File),
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
    pub async fn try_clone(&self) -> FsResult<Self> {
        match self {
            File::Real(file) => file.try_clone().await.map(File::from),
            File::Model(file) => Ok(file.clone().into()),
        }
    }

    pub async fn read(&mut self, buf: &mut [u8], offset: usize) -> FsResult<usize> {
        match self {
            File::Real(file) => file.read(buf, offset).await,
            File::Model(file) => file.read(buf, offset).await,
        }
    }

    pub async fn write(&mut self, buf: &[u8], offset: usize) -> FsResult<usize> {
        match self {
            File::Real(file) => file.write(buf, offset).await,
            File::Model(file) => file.write(buf, offset).await,
        }
    }

    pub async fn open(name: impl Into<String>) -> FsResult<Self> {
        if is_real() {
            real::File::open(name).await.map(File::from)
        } else {
            model::File::open(name).map(File::from)
        }
    }

    pub async fn create(name: impl Into<String>) -> FsResult<Self> {
        if is_real() {
            real::File::create(name).await.map(File::from)
        } else {
            model::File::create(name).map(File::from)
        }
    }

    pub async fn delete(name: impl Into<String>) -> FsResult<()> {
        if is_real() {
            real::File::delete(name).await
        } else {
            model::File::delete(name)
        }
    }
}
