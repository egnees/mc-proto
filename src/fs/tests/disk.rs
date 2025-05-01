use std::{cell::RefCell, pin::pin, rc::Rc, time::Duration};

use futures::executor::block_on;

use crate::{
    event::time::TimeSegment,
    fs::{
        disk::Disk,
        event::{FsEventKind, FsEventOutcome},
    },
};

use super::instant::InstantRegister;

////////////////////////////////////////////////////////////////////////////////

#[test]
fn one_request() {
    let reg = Rc::new(RefCell::new(InstantRegister::default()));
    let mut disk = Disk::new(
        reg,
        TimeSegment::new(Duration::from_millis(100), Duration::from_millis(200)),
        "node".into(),
        20,
    );
    let waiter = disk.enqueue_request(
        "proc".into(),
        FsEventKind::Read {
            file: "f1".into(),
            offset: 0,
            len: 5,
        },
    );
    let f = waiter.wait::<FsEventOutcome>();
    let f = pin!(f);
    let result = block_on(f).unwrap();
    assert!(result.is_ok());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn capacity() {
    let reg = Rc::new(RefCell::new(InstantRegister::default()));
    let mut disk = Disk::new(
        reg,
        TimeSegment::new(Duration::from_millis(100), Duration::from_millis(200)),
        "node".into(),
        20,
    );

    let kind = FsEventKind::Write {
        file: "f1".into(),
        offset: 0,
        len: 19,
    };

    let waiter = disk.enqueue_request("proc".into(), kind.clone());
    let f = waiter.wait::<FsEventOutcome>();
    let f = pin!(f);
    let result = block_on(f).unwrap();
    assert!(result.is_ok());

    disk.on_request_completed("proc".into(), kind, result);

    let kind = FsEventKind::Write {
        file: "f2".into(),
        offset: 0,
        len: 2,
    };
    let waiter = disk.enqueue_request("proc".into(), kind.clone());
    let f = waiter.wait::<FsEventOutcome>();
    let f = pin!(f);
    let result = block_on(f).unwrap();
    assert!(result.is_err());
    disk.on_request_completed("proc".into(), kind, result);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn capacity2() {
    let reg = Rc::new(RefCell::new(InstantRegister::default()));
    let mut disk = Disk::new(
        reg,
        TimeSegment::new(Duration::from_millis(100), Duration::from_millis(200)),
        "node".into(),
        20,
    );

    let kind = FsEventKind::Write {
        file: "f1".into(),
        offset: 0,
        len: 19,
    };
    let waiter = disk.enqueue_request("proc".into(), kind.clone());
    let f = waiter.wait::<FsEventOutcome>();
    let f = pin!(f);
    let result = block_on(f).unwrap();
    assert!(result.is_ok());
    disk.on_request_completed("proc".into(), kind, result);

    let kind = FsEventKind::Write {
        file: "f2".into(),
        offset: 0,
        len: 2,
    };

    let waiter = disk.enqueue_request("proc".into(), kind.clone());
    let f = waiter.wait::<FsEventOutcome>();
    let f = pin!(f);
    let result = block_on(f).unwrap();
    assert!(result.is_err());
    disk.on_request_completed("proc".into(), kind, result);

    disk.file_deleted(19);

    let kind = FsEventKind::Write {
        file: "f2".into(),
        offset: 0,
        len: 2,
    };
    let waiter = disk.enqueue_request("proc".into(), kind.clone());
    let f = waiter.wait::<FsEventOutcome>();
    let f = pin!(f);
    let result = block_on(f).unwrap();
    assert!(result.is_ok());
    disk.on_request_completed("proc".into(), kind, result);
}
