use std::{io::SeekFrom, path::PathBuf};

use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::{real::context::Context, FsError, FsResult};

////////////////////////////////////////////////////////////////////////////////

pub struct File {
    file: tokio::fs::File,
}

impl File {
    pub async fn try_clone(&self) -> FsResult<Self> {
        self.file
            .try_clone()
            .await
            .map_err(|_| FsError::FileNotAvailable)
            .map(|file| Self { file })
    }

    pub async fn read(&mut self, buf: &mut [u8], offset: usize) -> FsResult<usize> {
        self.file
            .seek(SeekFrom::Start(offset as u64))
            .await
            .map_err(|e| {
                println!("{e:?}");
                FsError::FileNotAvailable
            })?;
        self.file
            .read(buf)
            .await
            .map_err(|_| FsError::FileNotAvailable)
    }

    pub async fn write(&mut self, buf: &[u8], offset: usize) -> FsResult<usize> {
        self.file
            .seek(SeekFrom::Start(offset as u64))
            .await
            .map_err(|_| FsError::FileNotAvailable)?;
        self.file
            .write_all(buf)
            .await
            .map_err(|_| FsError::FileNotAvailable)?;
        self.file
            .shutdown()
            .await
            .map_err(|_| FsError::FileNotAvailable)?;
        Ok(buf.len())
    }

    pub async fn create(name: impl Into<String>) -> FsResult<Self> {
        let mount_dir = Context::current().mount_dir();
        let mut path = PathBuf::from(mount_dir);
        path.push(name.into());
        let file_path = path.clone().to_string_lossy().to_string();
        let file = tokio::fs::File::create_new(path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::AlreadyExists => FsError::FileAlreadyExists { file: file_path },
                _ => FsError::StorageNotAvailable,
            })?;
        Ok(Self { file })
    }

    pub async fn open(name: impl Into<String>) -> FsResult<Self> {
        let mount_dir = Context::current().mount_dir();
        let mut path = PathBuf::from(mount_dir);
        path.push(name.into());
        let file_path = path.clone().into_os_string().into_string().unwrap();
        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(false)
            .open(path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => FsError::FileNotFound { file: file_path },
                _ => FsError::StorageNotAvailable,
            })?;
        Ok(Self { file })
    }

    pub async fn delete(name: impl Into<String>) -> FsResult<()> {
        let mount_dir = Context::current().mount_dir();
        let mut path = PathBuf::from(mount_dir);
        path.push(name.into());
        let file_path = path.clone().into_os_string().into_string().unwrap();
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => FsError::FileNotFound { file: file_path },
                _ => FsError::StorageNotAvailable,
            })
    }
}
