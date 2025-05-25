use std::future::Future;

use smol::future::FutureExt;

use crate::{model, real};

////////////////////////////////////////////////////////////////////////////////

pub enum JoinHandle<T> {
    Sim(model::JoinHandle<T>),
    Real(real::JoinHandle<T>),
}

impl<T> JoinHandle<T> {
    pub fn abort(&mut self) {
        match self {
            JoinHandle::Sim(sim) => sim.abort(),
            JoinHandle::Real(real) => real.abort(),
        }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Option<T>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &mut *self {
            JoinHandle::Real(real) => real.poll(cx),
            JoinHandle::Sim(sim) => sim.poll(cx).map(|r| r.ok()),
        }
    }
}
