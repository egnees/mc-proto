use std::future::Future;

use smol::future::FutureExt;

////////////////////////////////////////////////////////////////////////////////

pub enum Timer {
    Sim(crate::timer::Timer),
    Real(crate::real::Timer),
}

impl Future for Timer {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &mut *self {
            Timer::Real(timer) => timer.poll(cx),
            Timer::Sim(timer) => timer.poll(cx),
        }
    }
}
