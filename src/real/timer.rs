use std::future::Future;

use smol::future::FutureExt;

////////////////////////////////////////////////////////////////////////////////

pub struct Timer {
    task: tokio::task::JoinHandle<()>,
}

impl Future for Timer {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.task.poll(cx).map(|e| e.unwrap())
    }
}

impl Timer {
    pub(crate) fn new(task: tokio::task::JoinHandle<()>) -> Self {
        Self { task }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
