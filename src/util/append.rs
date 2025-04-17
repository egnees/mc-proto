use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    task::{Poll, Waker},
};

use futures::future::poll_fn;

////////////////////////////////////////////////////////////////////////////////

struct SharedState {
    buffer: VecDeque<u8>,
    waker: Option<Waker>,
    sender_alive: bool,
    receiver_alive: bool,
}

////////////////////////////////////////////////////////////////////////////////

pub struct Sender {
    shared: Rc<RefCell<SharedState>>,
}

impl Sender {
    pub fn send(&self, buf: &[u8]) -> bool {
        if !self.shared.borrow().receiver_alive {
            return false;
        }
        if let Some(waker) = {
            let mut shared = self.shared.borrow_mut();
            shared.buffer.extend(buf.iter());
            shared.waker.take()
        } {
            waker.wake();
        }
        true
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        let mut shared = self.shared.borrow_mut();
        shared.sender_alive = false;
        if let Some(waker) = shared.waker.take() {
            waker.wake();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Receiver {
    shared: Rc<RefCell<SharedState>>,
}

impl Receiver {
    pub async fn recv(&mut self, buf: &mut [u8]) -> Option<usize> {
        poll_fn(|cx| {
            let mut state = self.shared.borrow_mut();
            if buf.is_empty() || !state.buffer.is_empty() {
                let len = buf.len().min(state.buffer.len());
                for e in buf.iter_mut().take(len) {
                    *e = state.buffer.pop_front().unwrap();
                }
                Poll::Ready(Some(len))
            } else if !state.sender_alive {
                Poll::Ready(None)
            } else {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        })
        .await
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.shared.borrow_mut().receiver_alive = false;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn mpsc_channel() -> (Sender, Receiver) {
    let shared = SharedState {
        buffer: VecDeque::new(),
        waker: None,
        sender_alive: true,
        receiver_alive: true,
    };
    let shared = Rc::new(RefCell::new(shared));
    let sender = Sender {
        shared: shared.clone(),
    };
    let receiver = Receiver { shared };
    (sender, receiver)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use tokio::task::{yield_now, LocalSet};

    use crate::util::append::mpsc_channel;

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn basic() {
        let rt = LocalSet::new();
        let (sender, mut receiver) = mpsc_channel();
        let sender = Rc::new(sender);
        rt.spawn_local({
            let sender = sender.clone();
            async move {
                let send_result = sender.send("hello".as_bytes());
                assert!(send_result);
            }
        });
        rt.spawn_local(async move {
            let mut buf1 = [0u8; 10];
            let bytes1 = receiver.recv(&mut buf1).await.unwrap();
            assert_eq!(bytes1, 5);
            assert_eq!(&buf1[..5], "hello".as_bytes());

            let bytes2 = receiver.recv(&mut buf1[..3]).await.unwrap();
            assert_eq!(bytes2, 3);
            assert_eq!(&buf1[..3], "333".as_bytes());

            let bytes3 = receiver.recv(&mut buf1).await.unwrap();
            assert_eq!(bytes3, 2);
            assert_eq!(&buf1[..2], "22".as_bytes());
        });
        rt.spawn_local(async move {
            sender.send("33322".as_bytes());
        });
        rt.await;
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn append() {
        let rt = LocalSet::new();
        let (sender, mut receiver) = mpsc_channel();
        rt.spawn_local(async move {
            sender.send("123".as_bytes());
            sender.send("456".as_bytes());
            yield_now().await;
            sender.send("111111".as_bytes());
            sender.send("111".as_bytes());
        });
        rt.spawn_local(async move {
            let mut buf = [0u8; 100];
            let bytes = receiver.recv(&mut buf).await.unwrap();
            assert_eq!(bytes, 6);
            let bytes = receiver.recv(&mut buf).await.unwrap();
            assert_eq!(bytes, 9);
        });
        rt.await;
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn sender_drops() {
        let (sender, mut receiver) = mpsc_channel();
        drop(sender);
        let recv_result = receiver.recv(&mut [0u8; 10]).await;
        assert!(recv_result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn receiver_drops() {
        let (sender, receiver) = mpsc_channel();
        drop(receiver);
        let send_result = sender.send("123".as_bytes());
        assert!(!send_result);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn many_senders() {
        let senders = 10;
        let (sender, mut receiver) = mpsc_channel();
        let sender = Rc::new(sender);
        for i in 0..senders {
            let sender = sender.clone();
            sender.send(i.to_string().as_bytes());
            drop(sender);
        }

        let mut buf = [0u8; 100];
        let bytes = receiver.recv(&mut buf).await.unwrap();
        assert_eq!(bytes, senders);
        assert_eq!(&buf[..bytes], "0123456789".as_bytes());
        drop(sender);
        let result = receiver.recv(&mut buf).await;
        assert!(result.is_none());
    }
}
