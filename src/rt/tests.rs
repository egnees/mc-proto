use std::{cell::RefCell, rc::Rc};

use tokio::sync::{mpsc, oneshot};

use crate::rt::spawn;

use super::{Runtime, RuntimeHandle};

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn basic() {
    let rt = Runtime::default();
    let handle = rt.handle();
    let (sender, receiver) = oneshot::channel();
    let (tx, rx) = oneshot::channel();

    rt.spawn(async move {
        let got = receiver.await.unwrap();
        assert_eq!(got, 1);
        tx.send(true).unwrap();
    });

    rt.spawn(async move {
        handle.spawn(async move {
            sender.send(1).unwrap();
        });
    });

    rt.process_tasks();

    let got = rx.await.unwrap();
    assert_eq!(got, true);
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn current() {
    let rt = Runtime::default();
    let (rx, tx) = oneshot::channel();

    rt.spawn(async {
        let handle = RuntimeHandle::current();
        handle.spawn(async move {
            rx.send(true).unwrap();
        });
    });

    rt.process_tasks();

    let got = tx.await.unwrap();
    assert_eq!(got, true);
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn inner_runtime() {
    let r1 = Runtime::default();
    let (rx, tx) = oneshot::channel();
    let (rx1, tx1) = oneshot::channel();

    r1.spawn(async {
        RuntimeHandle::current().spawn(async move {
            let r2 = Runtime::default();

            r2.spawn(async move {
                spawn(async move {
                    rx.send(true).unwrap();
                });
            });

            let processed = r2.process_tasks();
            assert!(processed > 0);

            let result = tx.await.unwrap();
            assert_eq!(result, true);

            r2.spawn(async move {
                spawn(async move {
                    rx1.send(true).unwrap();
                });
            });

            let processed = r2.process_tasks();
            assert!(processed > 0);
        })
    });

    r1.process_tasks();

    let result = tx1.await.unwrap();
    assert_eq!(result, true);
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn join_handle() {
    let rt = Runtime::default();
    let (tx, mut rx) = mpsc::channel(1024);

    let handle = rt.spawn(async {
        let task1 = spawn(async move { rx.recv().await.unwrap() });
        let task = spawn(async move {
            spawn(async move {
                tx.send(true).await.unwrap();
            });
            task1.await.unwrap()
        });
        task.await.unwrap()
    });

    let processed = rt.process_tasks();
    assert!(processed > 0);

    let r = handle.await.unwrap();
    assert_eq!(r, true);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
#[should_panic]
fn current_2() {
    let rt = Runtime::default();
    rt.spawn(async {
        let handle = RuntimeHandle::current();
        handle.spawn(async {});
    });
    rt.process_tasks();
    RuntimeHandle::current();
}
