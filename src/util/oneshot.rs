use std::{cell::RefCell, future::Future, rc::Rc};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Eq, PartialEq)]
pub enum RecvError {
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

pub struct Sender<T> {
    shared: Rc<RefCell<SharedState<T>>>,
}

impl<T> Sender<T> {
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
            SharedState::ReceiverWait(_) => unreachable!(),
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
    use crate::runtime;

    use super::*;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let (tx, rx) = channel::<i32>();
        let rt = runtime::Runtime::default();
        rt.spawn(async move {
            let x = rx.await.unwrap();
            assert_eq!(x, 2);
        });
        rt.spawn(async move {
            let result = tx.send(2);
            assert!(result.is_ok());
        });
        let proc = rt.process_tasks();
        assert!(proc > 2);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn drop_sender() {
        let (tx, rx) = channel::<i32>();
        let rt = runtime::Runtime::default();
        rt.spawn(async move {
            let result = rx.await;
            assert_eq!(result, Err(RecvError::SenderDropped));
        });
        rt.spawn(async move {
            drop(tx);
        });
        let proc = rt.process_tasks();
        assert!(proc > 2);
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
