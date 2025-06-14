//! Provides single threaded async oneshot channel abstraction.

use std::{cell::RefCell, future::Future, rc::Rc};

////////////////////////////////////////////////////////////////////////////////

/// Represents receive error.
#[derive(Debug, Eq, PartialEq)]
pub enum RecvError {
    /// Sender was dropped
    SenderDropped,
}

////////////////////////////////////////////////////////////////////////////////

enum SharedState<T> {
    Initial,
    ReceiverWait(std::task::Waker),
    SenderSent(T),
    SenderDropped,
    ReceiverDropped,
}

////////////////////////////////////////////////////////////////////////////////

/// Represents oneshot sender.
pub struct Sender<T> {
    shared: Rc<RefCell<SharedState<T>>>,
}

impl<T> Sender<T> {
    /// Allows to send into channel
    pub fn send(self, value: T) -> Result<(), T> {
        if let SharedState::ReceiverDropped = *self.shared.borrow() {
            return Err(value);
        }
        self.set_state(SharedState::SenderSent(value));
        Ok(())
    }

    fn set_state(&self, mut state: SharedState<T>) {
        let prev = &mut *self.shared.borrow_mut();
        std::mem::swap(&mut state, prev);
        if let SharedState::ReceiverWait(waker) = state {
            waker.wake();
        }
    }

    /// Check if receiver is alive.
    pub fn has_receiver(&self) -> bool {
        let state = self.shared.borrow();
        !matches!(*state, SharedState::<T>::ReceiverDropped)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if let SharedState::SenderSent(_) = &*self.shared.borrow() {
            return;
        }
        self.set_state(SharedState::SenderDropped);
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents oneshot receiver
pub struct Receiver<T> {
    shared: Rc<RefCell<SharedState<T>>>,
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, RecvError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = SharedState::ReceiverWait(cx.waker().clone());
        let prev = &mut *self.shared.borrow_mut();
        std::mem::swap(&mut state, prev);
        match state {
            SharedState::Initial => std::task::Poll::Pending,
            SharedState::ReceiverWait(_) => std::task::Poll::Pending,
            SharedState::SenderSent(value) => std::task::Poll::Ready(Ok(value)),
            SharedState::SenderDropped => std::task::Poll::Ready(Err(RecvError::SenderDropped)),
            SharedState::ReceiverDropped => unreachable!(),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        *self.shared.borrow_mut() = SharedState::ReceiverDropped;
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to make oneshot channel.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Rc::new(RefCell::new(SharedState::Initial));
    let sender = Sender {
        shared: shared.clone(),
    };
    let receiver = Receiver { shared };
    (sender, receiver)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use tokio::task::LocalSet;

    use super::*;

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn basic() {
        let (tx, rx) = channel::<i32>();
        let rt = LocalSet::new();
        rt.spawn_local(async move {
            let x = rx.await.unwrap();
            assert_eq!(x, 2);
        });
        rt.spawn_local(async move {
            let result = tx.send(2);
            assert!(result.is_ok());
        });
        rt.await;
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn drop_sender() {
        let (tx, rx) = channel::<i32>();
        let rt = LocalSet::new();
        rt.spawn_local(async move {
            let result = rx.await;
            assert_eq!(result, Err(RecvError::SenderDropped));
        });
        rt.spawn_local(async move {
            drop(tx);
        });
        rt.await;
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn drop_receiver() {
        let (tx, rx) = channel::<i32>();
        drop(rx);
        let result = tx.send(2);
        assert_eq!(result, Err(2));
    }
}
