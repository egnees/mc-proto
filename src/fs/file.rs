use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::sim::context::Context;

use super::{
    error::FsError,
    event::{FsEventKind, FsEventOutcome},
    manager::FsManagerHandle,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Hash)]
pub struct FileContent(Vec<u8>);

impl FileContent {
    fn read(&self, offset: usize, buf: &mut [u8]) {
        let len = buf.len();
        buf.copy_from_slice(&self.0[offset..offset + len]);
    }

    fn write(&mut self, offset: usize, buf: &[u8]) {
        let len = buf.len();
        if offset + len > self.0.len() {
            self.0.extend((0..offset + len - self.0.len()).map(|_| 0));
        }
        self.0.as_mut_slice()[offset..offset + len].copy_from_slice(buf);
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct File {
    content: Weak<RefCell<FileContent>>,
    fs: FsManagerHandle,
    name: String,
    pub(crate) owner_proc: String,
}

impl File {
    fn content(&self) -> Result<Rc<RefCell<FileContent>>, FsError> {
        if !self.fs.available() {
            return Err(FsError::StorageNotAvailable);
        }
        self.content.upgrade().ok_or(FsError::FileNotAvailable)
    }

    pub async fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, FsError> {
        let content = self.content()?;
        let residual = content.borrow().0.len().saturating_sub(offset);
        let len = buf.len().min(residual);
        let event = FsEventKind::Read {
            file: self.name.clone(),
            offset,
            len,
        };
        let waiter = self.fs.register_async_file_event(self, event.clone())?;
        let result = waiter
            .wait::<FsEventOutcome>()
            .await
            .unwrap_or(Err(FsError::StorageNotAvailable));

        self.fs.register_event_happen(self, event, result.clone());

        if result.is_ok() {
            content.borrow().read(offset, &mut buf[..len]);
        }

        result.map(|_| len)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn write(&self, buf: &[u8], offset: usize) -> Result<usize, FsError> {
        let content = self.content()?;
        let len = buf.len();
        let event = FsEventKind::Write {
            file: self.name.clone(),
            offset,
            len,
        };
        let waiter = self.fs.register_async_file_event(self, event.clone())?;
        let result = waiter
            .wait::<FsEventOutcome>()
            .await
            .unwrap_or(Err(FsError::StorageNotAvailable));

        self.fs.register_event_happen(self, event, result.clone());

        if result.is_ok() {
            content.borrow_mut().write(offset, buf);
        }

        result.map(|_| len)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn open_file(
        owner_proc: String,
        name: String,
        fs: FsManagerHandle,
    ) -> Result<File, FsError> {
        let content = fs.open_file(owner_proc.clone(), name.clone())?;
        Ok(File {
            content,
            fs,
            name,
            owner_proc,
        })
    }

    pub fn open(name: impl Into<String>) -> Result<File, FsError> {
        let ctx = Context::current();
        let proc = ctx.proc.address().process;
        let fs = ctx.fs.ok_or(FsError::StorageNotAvailable)?;
        Self::open_file(proc, name.into(), fs)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn create_file(
        owner_proc: String,
        name: String,
        fs: FsManagerHandle,
    ) -> Result<File, FsError> {
        let content = fs.create_file(owner_proc.clone(), name.clone())?;
        Ok(File {
            content,
            fs,
            name,
            owner_proc,
        })
    }

    pub fn create(name: impl Into<String>) -> Result<File, FsError> {
        let ctx = Context::current();
        let proc = ctx.proc.address().process;
        let fs = ctx.fs.ok_or(FsError::StorageNotAvailable)?;
        Self::create_file(proc, name.into(), fs)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn delete_file(proc: String, name: String, fs: FsManagerHandle) -> Result<(), FsError> {
        fs.delete_file(proc, name)
    }

    pub fn delete(name: impl Into<String>) -> Result<(), FsError> {
        let ctx = Context::current();
        let proc = ctx.proc.address().process;
        let fs = ctx.fs.ok_or(FsError::StorageNotAvailable)?;
        Self::delete_file(proc, name.into(), fs)
    }

    pub fn size(&self) -> Result<usize, FsError> {
        Ok(self.content()?.borrow().0.len())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::FileContent;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn file_content() {
        let mut content = FileContent::default();
        let mut buf = [0u8; 100];

        content.write(0, "hello".as_bytes());
        content.read(0, &mut buf[..5]);
        assert_eq!(&buf[..5], "hello".as_bytes());

        content.write(2, "abcdefg".as_bytes());
        content.read(1, &mut buf[..8]);
        assert_eq!(&buf[..8], "eabcdefg".as_bytes());
    }
}
