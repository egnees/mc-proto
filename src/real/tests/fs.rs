use crate::{send_local, spawn, Address, File, HashType, Process, RealNode};

////////////////////////////////////////////////////////////////////////////////

pub struct Proc1 {}

impl Process for Proc1 {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, file_name: String) {
        spawn(async move {
            let mut file = File::create(&file_name).await.unwrap();
            file.write("hello".as_bytes(), 0).await.unwrap();

            let mut buf = [0u8; 5];
            let bytes = file.read(&mut buf, 0).await.unwrap();
            assert_eq!(bytes, 5);
            assert_eq!(&buf[..bytes], "hello".as_bytes());

            File::delete(&file_name).await.unwrap();

            send_local("done");
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

fn test_scenario(proc: impl Process, file_name: impl Into<String>) {
    let mut node = RealNode::new(
        "node",
        123,
        Default::default(),
        std::env::temp_dir().to_string_lossy(),
    );
    let (sender, mut receiver) = node.add_proc("proc", proc).unwrap();
    sender.send(file_name);
    node.block_on(async move {
        let data = receiver.recv::<String>().await.unwrap();
        assert_eq!(data, "done");
    });
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic() {
    test_scenario(Proc1 {}, "proc1.txt");
}
