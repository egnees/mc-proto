//! Common async primitivies.

use std::future::Future;

use smol::future::FutureExt;

use crate::{model, real};

////////////////////////////////////////////////////////////////////////////////

/// Represents handle of the spawned async acitivity
/// (returned by spawn [`crate::spawn`]);
pub enum JoinHandle<T> {
    /// Real handle
    Real(real::JoinHandle<T>),

    /// Handle for the async activity in system model.
    Model(model::JoinHandle<T>),
}

impl<T> JoinHandle<T> {
    /// Allows to cancel async activity.
    pub fn abort(&mut self) {
        match self {
            JoinHandle::Model(sim) => sim.abort(),
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
            JoinHandle::Model(sim) => sim.poll(cx).map(|r| r.ok()),
        }
    }
}
