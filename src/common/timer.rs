//! Allows to set timers.

use std::future::Future;

use smol::future::FutureExt;

use crate::{model, real};

////////////////////////////////////////////////////////////////////////////////

/// Represents timer.
pub enum Timer {
    /// In the model of system
    Model(model::Timer),

    /// In the real environment
    Real(real::Timer),
}

impl Future for Timer {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &mut *self {
            Timer::Real(timer) => timer.poll(cx),
            Timer::Model(timer) => timer.poll(cx),
        }
    }
}
