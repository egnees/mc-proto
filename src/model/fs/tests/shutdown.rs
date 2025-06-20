use std::time::Duration;

use crate::model::fs::tests::instant::make_shared_instant;

use crate::model::fs::{file::File, manager::FsManager};
use crate::FsError;

////////////////////////////////////////////////////////////////////////////////

#[test]
fn shutdown_basic() {
    let reg = make_shared_instant();
    let manager = FsManager::new(
        reg.clone(),
        "node".into(),
        Duration::from_millis(20),
        Duration::from_millis(100),
        5,
    );
    let handle = manager.handle();

    let mut file = File::create_file("proc".into(), "f1".into(), handle.clone()).unwrap();
    handle.shutdown();

    // on create
    let result = File::create_file("proc".into(), "f2".into(), handle.clone());
    assert!(result.is_err_and(|e| e == FsError::StorageNotAvailable));

    // on open
    let result = File::open_file("proc".into(), "f1".into(), handle.clone());
    assert!(result.is_err_and(|e| e == FsError::StorageNotAvailable));

    // on delete
    let result = File::delete_file("proc".into(), "f1".into(), handle.clone());
    assert!(result.is_err_and(|e| e == FsError::StorageNotAvailable));

    // check read and write
    let rt = smol::LocalExecutor::new();
    let f = rt.run(async move {
        let result = file.read(&mut [0u8; 100], 0).await;
        assert!(result.is_err_and(|e| e == FsError::StorageNotAvailable));

        let result = file.write("hello".as_bytes(), 0).await;
        assert!(result.is_err_and(|e| e == FsError::StorageNotAvailable));
    });

    futures::executor::block_on(f);

    // raise
    handle.raise();

    let result = File::create_file("proc".into(), "f1".into(), handle.clone());
    assert!(result.is_err_and(|e| e == FsError::FileAlreadyExists { file: "f1".into() }));

    File::delete_file("proc".into(), "f1".into(), handle.clone()).unwrap();

    let mut file = File::create_file("proc".into(), "f1".into(), handle.clone()).unwrap();
    let f = rt.run(async move {
        let result = file.write("hello".as_bytes(), 0).await.unwrap();
        assert_eq!(result, 5);

        let mut buf = [0u8; 10];
        let bytes = file.read(&mut buf, 0).await.unwrap();
        assert_eq!(bytes, 5);
        assert_eq!(&buf[..5], "hello".as_bytes());
    });

    futures::executor::block_on(f);
}
