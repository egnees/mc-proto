use std::time::Duration;

use crate::{
    event::time::Time,
    fs::{error::FsError, event::FsEventOutcome, file::File, manager::FsManager},
    util,
};

use super::{delayed::make_delayed_register, instant::make_shared_instant};

////////////////////////////////////////////////////////////////////////////////

#[test]
fn works() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg.clone(), "node".into(), delays, 1024);
    let handle = manager.handle();
    let file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();
    let file1 = file.clone();

    let rt = smol::LocalExecutor::new();
    let (s, r) = util::oneshot::channel::<i32>();
    rt.spawn(async move {
        file.write("hello".as_bytes(), 0).await.unwrap();
        s.send(0).unwrap();
    })
    .detach();

    let f = rt.run(async move {
        r.await.unwrap();
        let mut buf = [0u8; 100];
        let bytes = file1.read(&mut buf, 0).await.unwrap();
        assert_eq!(bytes, 5);
        assert_eq!(&buf[..5], "hello".as_bytes());
    });

    futures::executor::block_on(f);

    println!("{}", reg.borrow().log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn capacity_exceed() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg.clone(), "node".into(), delays, 5);
    let handle = manager.handle();
    let file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();

    let rt = smol::LocalExecutor::new();
    let f = rt.run(async move {
        file.write("1234".as_bytes(), 0).await.unwrap();
        file.write("5".as_bytes(), 4).await.unwrap();
        let e = file.write("6".as_bytes(), 0).await;
        assert!(e.is_err());
    });

    futures::executor::block_on(f);

    println!("{}", reg.borrow().log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn delete_works() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg.clone(), "node".into(), delays, 5);
    let handle = manager.handle();

    let file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();
    File::delete_file("proc".into(), "f1".into(), handle.clone()).unwrap();

    let rt = smol::LocalExecutor::new();
    let f = rt.run(async move {
        let result = file.write("hello".as_bytes(), 0).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FsError::FileNotAvailable);
    });

    futures::executor::block_on(f);

    println!("{}", reg.borrow().log);
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn already_exists() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg, "node".into(), delays, 5);
    let handle = manager.handle();

    let _file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();
    File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();
}

////////////////////////////////////////////////////////////////////////////////

#[should_panic]
#[test]
fn delete_not_existant() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg, "node".into(), delays, 5);
    let handle = manager.handle();

    let _file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();
    File::delete_file("proc".into(), "f2".into(), handle.clone()).unwrap();
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn open() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg.clone(), "node".into(), delays, 5);
    let handle = manager.handle();

    let file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();

    let rt = smol::LocalExecutor::new();
    let f = rt.run(async move {
        let bytes = file.write("hello".as_bytes(), 0).await.unwrap();
        assert_eq!(bytes, 5);
    });

    futures::executor::block_on(f);

    let file = File::open_file("f1".into(), "proc".into(), handle).unwrap();

    let f = rt.run(async move {
        let mut buf = [0u8; 100];
        let bytes = file.read(&mut buf, 0).await.unwrap();
        assert_eq!(bytes, 5);
        assert_eq!(&buf[..5], "hello".as_bytes());
    });

    futures::executor::block_on(f);

    println!("{}", reg.borrow().log);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn open_not_existant() {
    let reg = make_shared_instant();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg.clone(), "node".into(), delays, 5);
    let handle = manager.handle();

    let _file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();

    let result = File::open_file("f2".into(), "proc".into(), handle).inspect_err(|e| {
        assert_eq!(
            *e,
            FsError::FileNotFound {
                file: "f2".to_string()
            }
        )
    });
    assert!(result.is_err());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn concurrent_events() {
    let reg = make_delayed_register();
    let delays = Time::new_segment(Duration::from_millis(20), Duration::from_millis(100));
    let manager = FsManager::new(reg.clone(), "node".into(), delays, 100);
    let handle = manager.handle();

    let file = File::create_file("f1".into(), "proc".into(), handle.clone()).unwrap();

    let rt = smol::LocalExecutor::new();
    rt.spawn({
        let file = file.clone();
        async move {
            file.write("hello".as_bytes(), 0).await.unwrap();
        }
    })
    .detach();

    rt.spawn({
        let file = file.clone();
        async move {
            file.write("hello1".as_bytes(), 5).await.unwrap();
        }
    })
    .detach();

    loop {
        let result = rt.try_tick();
        if !result {
            break;
        }
    }

    assert_eq!(reg.borrow().events.len(), 1);
    let e = reg.borrow_mut().events.remove(0);
    e.0.invoke::<FsEventOutcome>(e.1).unwrap();

    loop {
        let result = rt.try_tick();
        if !result {
            break;
        }
    }

    assert_eq!(reg.borrow().events.len(), 1);
    let e = reg.borrow_mut().events.remove(0);
    e.0.invoke::<FsEventOutcome>(e.1).unwrap();

    loop {
        let result = rt.try_tick();
        if !result {
            break;
        }
    }

    rt.spawn(async move {
        if reg.borrow().events.is_empty() {
            tokio::task::yield_now().await;
        }
        let e = reg.borrow_mut().events.remove(0);
        e.0.invoke::<FsEventOutcome>(e.1).unwrap();
    })
    .detach();

    let f = rt.run(async move {
        let mut buf = [0u8; 100];
        let bytes = file.read(&mut buf, 0).await.unwrap();
        assert_eq!(bytes, 5 + 6);
        assert_eq!(&buf[..11], "hellohello1".as_bytes());
    });

    futures::executor::block_on(f);
}
