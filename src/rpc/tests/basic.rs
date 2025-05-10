use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    rpc::{registry::RpcRegistry, request::RpcRequest, response::RpcResponse},
    Address,
};

use super::instant::InstantRpcRegistry;

use crate::rpc::request::rpc_impl;

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
struct Request {
    x: i32,
}

impl From<&RpcRequest> for Request {
    fn from(value: &RpcRequest) -> Self {
        serde_json::from_slice(&value.content.as_slice()).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
struct Response {
    y: i32,
}

impl From<RpcResponse> for Response {
    fn from(value: RpcResponse) -> Self {
        serde_json::from_slice(&value.content.as_slice()).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic() {
    let reg = InstantRpcRegistry::default();
    let reg = Rc::new(RefCell::new(reg));
    let a1 = Address::new("n1", "p1");
    let a2 = Address::new("n2", "p2");
    let mut listener = reg.borrow_mut().register_listener(a2.clone()).unwrap();

    let rt = smol::LocalExecutor::new();
    rt.spawn(async move {
        let request = listener.listen().await;
        let ser: Request = (&request).into();
        request.reply(&Response { y: ser.x + 1 }).unwrap();
    })
    .detach();

    let f = rt.run(async move {
        let result: Response = rpc_impl::<Request>(a1, a2, reg, 0, &Request { x: 2 })
            .await
            .unwrap()
            .into();
        result
    });

    let result = futures::executor::block_on(f);
    assert_eq!(result.y, 3);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn listen_after_request() {
    let reg = InstantRpcRegistry::default();
    let reg = Rc::new(RefCell::new(reg));
    let a1 = Address::new("n1", "p1");
    let a2 = Address::new("n2", "p2");
    let mut listener = reg.borrow_mut().register_listener(a2.clone()).unwrap();

    let rt = smol::LocalExecutor::new();
    let f = rt.run(async move {
        let result: Response = rpc_impl::<Request>(a1, a2, reg, 0, &Request { x: 2 })
            .await
            .unwrap()
            .into();
        result
    });

    for _ in 0..100 {
        rt.try_tick();
    }

    rt.spawn(async move {
        let request = listener.listen().await;
        let ser: Request = (&request).into();
        request.reply(&Response { y: ser.x + 1 }).unwrap();
    })
    .detach();

    let result = futures::executor::block_on(f);
    assert_eq!(result.y, 3);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn double_listen() {
    let mut reg = InstantRpcRegistry::default();
    let a = Address::new("n1", "p1");
    let listener = reg.register_listener(a.clone()).unwrap();
    let result = reg.register_listener(a);
    assert!(result.is_err());
    drop(listener);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn drop_listener() {
    let mut reg = InstantRpcRegistry::default();
    let a = Address::new("n1", "p1");
    let listener = reg.register_listener(a.clone()).unwrap();
    drop(listener);
    reg.register_listener(a).unwrap();
}
