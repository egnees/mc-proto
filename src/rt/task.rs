use std::{future::Future, pin::Pin};

////////////////////////////////////////////////////////////////////////////////

pub type TaskId = usize;
pub type Task = Pin<Box<dyn Future<Output = ()>>>;

////////////////////////////////////////////////////////////////////////////////

/// Appears when `JoinHandle` has been dropped.
#[derive(Debug)]
pub struct JoinError {}

////////////////////////////////////////////////////////////////////////////////

pub struct JoinHandle<T> {
    task_id: TaskId,
    result: tokio::sync::oneshot::Receiver<T>,
}

////////////////////////////////////////////////////////////////////////////////

impl<T> JoinHandle<T> {
    pub fn new(task_id: TaskId, result: tokio::sync::oneshot::Receiver<T>) -> Self {
        Self { task_id, result }
    }

    pub fn id(&self) -> TaskId {
        self.task_id
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|h| &mut h.result) }
            .poll(cx)
            .map_err(|_| JoinError {})
    }
}
