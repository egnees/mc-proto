use std::{cell::RefCell, fmt::Display, rc::Rc, time::Instant};

use smol::{future, stream::StreamExt, LocalExecutor};
use tokio::task::LocalSet;

use crate::{
    runtime::Runtime,
    sim::proc::{ProcessHandle, ProcessState},
    tcp::stream::TcpSender,
    util::trigger::Trigger,
    Address, HashType, Process,
};

use crate::tcp::{
    error::TcpError, manager::TcpConnectionManager, packet::TcpPacket, registry::TcpRegistry,
    stream::TcpStream,
};

////////////////////////////////////////////////////////////////////////////////

pub enum LogEntry {
    Send(TcpPacket),
    Deliver(TcpPacket),
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogEntry::Send(p) => write!(f, "--> {}", p),
            LogEntry::Deliver(p) => write!(f, "<-- {}", p),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct InstantTcpRegister {
    log: Vec<LogEntry>,
    manager: TcpConnectionManager,
    next_stream_id: usize,
}

impl TcpRegistry for InstantTcpRegister {
    fn emit_packet(
        &mut self,
        _from: &Address,
        _to: &Address,
        packet: &TcpPacket,
        on_delivery: Trigger,
    ) -> Result<(), TcpError> {
        self.log.push(LogEntry::Send(packet.clone()));
        let _ = on_delivery.invoke::<Result<(), TcpError>>(Ok(()));
        Ok(())
    }

    fn emit_listen_request(&mut self, from: &Address, on_listen: Trigger) -> Result<(), TcpError> {
        self.manager.listen(from, on_listen)
    }

    fn emit_listen_to_request(
        &mut self,
        from: &Address,
        to: &Address,
        on_listen: Trigger,
    ) -> Result<(), TcpError> {
        self.manager.listen_to(from, to, on_listen)
    }

    fn emit_sender_dropped(&mut self, _sender: &mut TcpSender) {
        // do nothing
    }

    fn register_packet_delivery(
        &mut self,
        _from: &Address,
        _to: &Address,
        packet: &TcpPacket,
    ) -> Result<(), TcpError> {
        self.log.push(LogEntry::Deliver(packet.clone()));
        Ok(())
    }

    fn try_connect(
        &mut self,
        from: &Address,
        to: &Address,
        stream_id: usize,
        registry_ref: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        self.manager.connect(from, to, stream_id, registry_ref)
    }

    fn next_tcp_stream_id(&mut self) -> usize {
        let res = self.next_stream_id;
        self.next_stream_id += 1;
        res
    }
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn basic() {
    let rt = LocalSet::new();
    let a1: Address = "n1:p1".into();
    let a2: Address = "n2:p2".into();
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    rt.spawn_local({
        let a1 = a1.clone();
        let reg = reg.clone();
        async move {
            let mut stream = TcpStream::listen(a1, reg).await.unwrap();
            let mut bytes = [0u8; 10];
            let read = stream.recv(&mut bytes).await.unwrap();
            assert_eq!(read, "hello".len());
            assert_eq!(&bytes[..read], "hello".as_bytes())
        }
    });
    rt.spawn_local({
        let reg = reg.clone();
        async move {
            let stream = TcpStream::connect_addr(a2, a1, reg.clone()).await.unwrap();
            let bytes = stream.send("hello".as_bytes()).await.unwrap();
            assert_eq!(bytes, "hello".len());
        }
    });
    rt.await;
    for log in reg.borrow().log.iter() {
        println!("{}", log);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn no_listen() {
    let rt = LocalSet::new();
    let a1: Address = "n1:p1".into();
    let a2: Address = "n2:p2".into();
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    rt.spawn_local({
        let reg = reg.clone();
        let a1 = a1.clone();
        async move {
            let result = TcpStream::connect_addr(a2, a1, reg.clone()).await;
            assert!(result.is_err());
        }
    });
    rt.await;
    for log in reg.borrow().log.iter() {
        println!("{}", log);
    }
}

////////////////////////////////////////////////////////////////////////////////

async fn try_connect(me: Address, to: Address, reg: Rc<RefCell<InstantTcpRegister>>) -> TcpStream {
    loop {
        let con_result = TcpStream::connect_addr(me.clone(), to.clone(), reg.clone()).await;
        if let Ok(stream) = con_result {
            return stream;
        } else {
            tokio::task::yield_now().await;
        }
    }
}

async fn try_listen(me: Address, to: Address, reg: Rc<RefCell<InstantTcpRegister>>) -> TcpStream {
    loop {
        let listen_result = TcpStream::listen_to(me.clone(), to.clone(), reg.clone()).await;
        if let Ok(stream) = listen_result {
            return stream;
        } else {
            tokio::task::yield_now().await;
        }
    }
}

async fn make_connection_with_retry(
    me: Address,
    to: Address,
    reg: Rc<RefCell<InstantTcpRegister>>,
) -> TcpStream {
    tokio::select! {
        stream = try_connect(me.clone(), to.clone(), reg.clone()) => stream,
        stream = try_listen(me.clone(), to.clone(), reg.clone()) => stream,
    }
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn symmetric_connection() {
    let rt = LocalSet::new();
    let a1: Address = "n1:p1".into();
    let a2: Address = "n2:p2".into();
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    rt.spawn_local({
        let a1 = a1.clone();
        let a2 = a2.clone();
        let reg = reg.clone();
        async move {
            let mut stream = make_connection_with_retry(a1, a2, reg).await;
            let mut bytes = [0u8; 10];
            let send_bytes = stream.recv(&mut bytes).await.unwrap();
            assert_eq!(&bytes[..send_bytes], "hello".as_bytes());
        }
    });
    rt.spawn_local(async move {
        let stream = make_connection_with_retry(a2, a1, reg).await;
        let send_bytes = stream.send("hello".as_bytes()).await.unwrap();
        assert_eq!(send_bytes, "hello".len());
    });
    rt.await;
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn drop_works() {
    struct Proc {}

    impl Process for Proc {
        fn on_message(&mut self, _from: Address, _content: String) {
            unreachable!()
        }

        fn on_local_message(&mut self, _content: String) {
            unreachable!()
        }

        fn hash(&self) -> HashType {
            unreachable!()
        }
    }

    let rt = Runtime::default();
    let a1 = Address::new("n1", "p1");
    let a2 = Address::new("n2", "p2");
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    let proc = Proc {};
    let proc_state = ProcessState::new(Rc::new(RefCell::new(proc)), "n1:p1".into());
    let handle = ProcessHandle::new(&Rc::new(RefCell::new(proc_state)));
    rt.handle().spawn(
        {
            let a1 = a1.clone();
            let a2 = a2.clone();
            let reg = reg.clone();
            async move {
                tokio::task::yield_now().await;
                let _ = make_connection_with_retry(a1, a2, reg).await;
            }
        },
        handle.clone(),
    );
    rt.handle().spawn(
        async move {
            let mut stream = make_connection_with_retry(a2, a1, reg).await;
            let result = stream.recv(&mut [0u8; 10]).await;
            assert!(result.is_err());
            println!("success");
        },
        handle,
    );
    let tasks = rt.process_tasks();
    println!("tasks={tasks}");
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn drop_works_smol() {
    let rt = LocalExecutor::new();
    let a1 = Address::new("n1", "p1");
    let a2 = Address::new("n2", "p2");
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    rt.spawn({
        let a1 = a1.clone();
        let a2 = a2.clone();
        let reg = reg.clone();
        async move {
            let _ = make_connection_with_retry(a1, a2, reg).await;
        }
    })
    .detach();
    future::block_on(rt.run(async move {
        let mut stream = make_connection_with_retry(a2, a1, reg.clone()).await;
        let result = stream.recv(&mut [0u8; 10]).await;
        assert!(result.is_err());
        println!("success");
        println!("log:");
        for e in reg.borrow().log.iter() {
            println!("{e}");
        }
    }));
}

////////////////////////////////////////////////////////////////////////////////

async fn server(addr: Address, reg: Rc<RefCell<InstantTcpRegister>>, rt: Rc<LocalExecutor<'_>>) {
    loop {
        let stream = TcpStream::listen(addr.clone(), reg.clone()).await.unwrap();
        rt.spawn(echo_stream(stream)).detach();
    }
}

async fn echo_stream(mut stream: TcpStream) -> Result<(), TcpError> {
    let mut buf = [0u8; 4096];
    loop {
        let bytes = stream.recv(&mut buf).await?;
        stream.send(&buf[..bytes]).await?;
    }
}

async fn client(
    me: Address,
    server: Address,
    reg: Rc<RefCell<InstantTcpRegister>>,
) -> Result<(), TcpError> {
    let mut buf = [0u8; 4096];

    let msg1 = format!("hello from client {}", me);
    let msg2 = format!("hello from client {} [2]", me);

    let mut stream = try_connect(me, server, reg).await;

    stream.send(msg1.as_bytes()).await?;
    let bytes = stream.recv(&mut buf).await?;
    assert_eq!(&buf[..bytes], msg1.as_bytes());

    stream.send(msg2.as_bytes()).await?;
    let bytes = stream.recv(&mut buf).await?;
    assert_eq!(&buf[..bytes], msg2.as_bytes());

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn echo_server_one_client() {
    let rt = Rc::new(LocalExecutor::new());
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    let serv = Address::new("server", "p");
    rt.spawn(server(serv.clone(), reg.clone(), rt.clone()))
        .detach();
    let c = client("client:1".into(), serv, reg);
    let c = rt.run(c);
    let result = smol::future::block_on(c);
    assert!(result.is_ok());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn echo_server_10_clients() {
    let rt = LocalExecutor::new();
    let rt = Rc::new(rt);
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    let serv = Address::new("server", "p");
    let clients = 10;
    rt.spawn(server(serv.clone(), reg.clone(), rt.clone()))
        .detach();
    let clients = (0..clients)
        .map(|i| Address::new("client", i.to_string()))
        .map(|a| client(a, serv.clone(), reg.clone()));
    let mut tasks = Vec::new();
    rt.spawn_many(clients, &mut tasks);
    smol::future::block_on(rt.run(async move {
        smol::stream::iter(tasks)
            .then(|x| x)
            .collect::<Vec<_>>()
            .await
    }))
    .iter()
    .for_each(|r| assert!(r.is_ok()));
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn echo_server_100_clients() {
    let time = Instant::now();
    let rt = LocalExecutor::new();
    let rt = Rc::new(rt);
    let reg = Rc::new(RefCell::new(InstantTcpRegister::default()));
    let serv = Address::new("server", "p");
    let clients = 100;
    rt.spawn(server(serv.clone(), reg.clone(), rt.clone()))
        .detach();
    let clients = (0..clients)
        .map(|i| Address::new("client", i.to_string()))
        .map(|a| client(a, serv.clone(), reg.clone()));
    let mut tasks = Vec::new();
    rt.spawn_many(clients, &mut tasks);
    smol::future::block_on(rt.run(async move {
        smol::stream::iter(tasks)
            .then(|x| x)
            .collect::<Vec<_>>()
            .await
    }))
    .iter()
    .for_each(|r| assert!(r.is_ok()));
    let duration = time.elapsed();
    println!("Elapsed={}ms", duration.as_millis());
}
