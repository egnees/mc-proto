use std::{future::Future, pin::Pin};

////////////////////////////////////////////////////////////////////////////////

pub struct SendSyncWrapper<F>
where
    F: Future,
{
    f: F,
}

impl<F: Future> SendSyncWrapper<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Future for SendSyncWrapper<F>
where
    F: Future,
    F::Output: Send,
{
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|w| &mut w.f) }.poll(cx)
    }
}

unsafe impl<F: Future> Send for SendSyncWrapper<F> {}
// unsafe impl<F: Future> Sync for SendSyncWrapper<F> {}
