use std::{future::Future, pin::Pin};

use crate::{sim::proc::ProcessHandle, util};

use super::RuntimeHandle;

////////////////////////////////////////////////////////////////////////////////

pub type TaskId = usize;

pub struct Task {
    pub future: Pin<Box<dyn Future<Output = ()>>>,
    pub owner: ProcessHandle,
}

////////////////////////////////////////////////////////////////////////////////

/// Appears when `JoinHandle` has been dropped.
#[derive(Debug)]
pub struct JoinError {}

////////////////////////////////////////////////////////////////////////////////

pub struct JoinHandle<T> {
    task_id: TaskId,
    result: util::oneshot::Receiver<T>,
    rt: RuntimeHandle,
}

////////////////////////////////////////////////////////////////////////////////

impl<T> JoinHandle<T> {
    pub fn new(task_id: TaskId, result: util::oneshot::Receiver<T>, rt: RuntimeHandle) -> Self {
        Self {
            task_id,
            result,
            rt,
        }
    }

    pub fn id(&self) -> TaskId {
        self.task_id
    }

    pub fn abort(&self) {
        self.rt.cancel_task(self.task_id);
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
