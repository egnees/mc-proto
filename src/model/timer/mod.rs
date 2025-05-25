use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};

use registry::TimerRegistry;
use smol::future::FutureExt;

use crate::{util::oneshot, Address};

////////////////////////////////////////////////////////////////////////////////

pub mod manager;
pub mod registry;

////////////////////////////////////////////////////////////////////////////////

pub struct Timer {
    recv: oneshot::Receiver<()>,
    reg: Rc<RefCell<dyn TimerRegistry>>,
    id: usize,
    address: Address,
}

impl Timer {
    pub(crate) fn new(
        min_duration: Duration,
        max_duration: Duration,
        reg: Rc<RefCell<dyn TimerRegistry>>,
        with_sleep: bool,
        address: Address,
    ) -> Self {
        let (id, recv) = reg.borrow_mut().register_timer(
            min_duration,
            max_duration,
            with_sleep,
            address.clone(),
        );
        Self {
            recv,
            reg,
            id,
            address,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.recv.poll(cx).map(|r| assert!(r.is_ok()))
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.reg
            .borrow_mut()
            .cancel_timer(self.id, self.address.clone());
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn set_timer(duration: Duration) -> Timer {
    let cx = crate::model::context::Context::current();
    let reg = cx.event_manager.timer_registry();
    Timer::new(duration, duration, reg, false, cx.proc.address())
}

////////////////////////////////////////////////////////////////////////////////

pub fn set_random_timer(min_duration: Duration, max_duration: Duration) -> Timer {
    let cx = crate::model::context::Context::current();
    let reg = cx.event_manager.timer_registry();
    Timer::new(min_duration, max_duration, reg, false, cx.proc.address())
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
