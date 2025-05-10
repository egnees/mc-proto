use std::{cell::RefCell, collections::VecDeque, rc::Rc, task::Poll};

use futures::future::poll_fn;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
enum ChannelState {
    #[default]
    Idle,
    ReceiverWait(std::task::Waker),
    SenderDropped,
    ReceiverDropped,
}

////////////////////////////////////////////////////////////////////////////////

struct ChannelQueue<T> {
    queue: VecDeque<T>,
    state: ChannelState,
}

impl<T> ChannelQueue<T> {
    fn new() -> Self {
        Self {
            queue: Default::default(),
            state: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Sender<T> {
    queue: Rc<RefCell<ChannelQueue<T>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), T> {
        let mut s = self.queue.borrow_mut();
        if let ChannelState::ReceiverDropped = s.state {
            return Err(value);
        }

        s.queue.push_back(value);

        let mut state = ChannelState::Idle;
        std::mem::swap(&mut s.state, &mut state);
        if let ChannelState::ReceiverWait(w) = state {
            w.wake();
        }

        Ok(())
    }

    pub fn receiver_alive(&self) -> bool {
        !matches!(self.queue.borrow().state, ChannelState::ReceiverDropped)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut state = ChannelState::SenderDropped;
        std::mem::swap(&mut self.queue.borrow_mut().state, &mut state);
        if let ChannelState::ReceiverWait(w) = state {
            w.wake();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Receiver<T> {
    queue: Rc<RefCell<ChannelQueue<T>>>,
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        poll_fn(|cx| {
            let mut s = self.queue.borrow_mut();
            if let Some(x) = s.queue.pop_front() {
                return Poll::Ready(Some(x));
            }
            if let ChannelState::SenderDropped = s.state {
                return Poll::Ready(None);
            }
            s.state = ChannelState::ReceiverWait(cx.waker().clone());
            Poll::Pending
        })
        .await
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        std::mem::swap(
            &mut self.queue.borrow_mut().state,
            &mut ChannelState::ReceiverDropped,
        );
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_channel<T>() -> (Sender<T>, Receiver<T>) {
    let s = ChannelQueue::new();
    let s = Rc::new(RefCell::new(s));
    let sender = Sender { queue: s.clone() };
    let receiver = Receiver { queue: s };
    (sender, receiver)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::make_channel;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let (rx, mut tx) = make_channel::<i32>();
        rx.send(1).unwrap();
        rx.send(2).unwrap();
        rx.send(3).unwrap();

        let rt = smol::LocalExecutor::new();
        let f = rt.run(async move {
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 1);
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 2);
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 3);
            tx
        });
        let mut tx = futures::executor::block_on(f);

        let f = rt.run(async move {
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 4);
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 5);
        });
        rx.send(4).unwrap();
        rx.send(5).unwrap();
        futures::executor::block_on(f);

        let a = rx.send(123).unwrap_err();
        assert_eq!(a, 123);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn sender_drop() {
        let (rx, mut tx) = make_channel::<i32>();
        drop(rx);
        let result = futures::executor::block_on(tx.recv());
        assert!(result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn sender_drop_after_recv_blocked() {
        let (rx, mut tx) = make_channel::<i32>();
        let rt = smol::LocalExecutor::new();
        let f = rt.run(tx.recv());
        drop(rx);
        let result = futures::executor::block_on(f);
        assert!(result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn sender_drop_1() {
        let (rx, mut tx) = make_channel::<i32>();
        rx.send(1).unwrap();
        rx.send(2).unwrap();
        let rt = smol::LocalExecutor::new();
        let f = rt.run(async move {
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 1);
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 2);
            tx.recv().await
        });
        drop(rx);
        let result = futures::executor::block_on(f);
        assert!(result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn sender_drop_2() {
        let (rx, mut tx) = make_channel::<i32>();
        rx.send(1).unwrap();
        rx.send(2).unwrap();
        drop(rx);
        let rt = smol::LocalExecutor::new();
        let f = rt.run(async move {
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 1);
            let a = tx.recv().await.unwrap();
            assert_eq!(a, 2);
            tx.recv().await
        });
        let result = futures::executor::block_on(f);
        assert!(result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn recv_drop() {
        let (rx, tx) = make_channel::<i32>();
        drop(tx);
        rx.send(123).unwrap_err();
    }
}
