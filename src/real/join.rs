use std::future::Future;

use smol::future::FutureExt;

pub struct JoinHandle<T> {
    pub(crate) task: tokio::task::JoinHandle<T>,
}

impl<T> JoinHandle<T> {
    pub fn abort(&mut self) {
        self.task.abort();
    }

    pub fn is_finished(&self) -> bool {
        self.task.is_finished()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Option<T>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.task.poll(cx).map(|e| e.ok())
    }
}
