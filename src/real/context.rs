use std::{cell::RefCell, future::Future, time::Duration};

use serde::Serialize;

use crate::{Address, RpcError, RpcResult};

use super::{
    proc::ProcessHandle,
    rpc::{response::RpcResponse, RpcListener},
    JoinHandle, Timer,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Context {
    proc: ProcessHandle,
}

impl Context {
    pub(crate) fn new(proc: ProcessHandle) -> Self {
        Self { proc }
    }
}

////////////////////////////////////////////////////////////////////////////////

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

////////////////////////////////////////////////////////////////////////////////

impl Context {
    pub fn installed() -> bool {
        CONTEXT.with(|c| c.borrow().is_some())
    }

    pub fn current() -> Context {
        CONTEXT.with(|c| {
            c.borrow()
                .as_ref()
                .expect("context is not installed")
                .clone()
        })
    }

    fn install(ctx: Context) {
        CONTEXT.with(|c| *c.borrow_mut() = Some(ctx));
    }

    fn reset() {
        CONTEXT.with(|c| *c.borrow_mut() = None);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn spawn<F>(&self, f: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send,
    {
        self.proc.node.spawn(f)
    }

    pub fn set_timer(&self, duration: Duration) -> Timer {
        self.proc.node.set_timer(duration)
    }

    pub fn set_random_timer(&self, min_duration: Duration, max_duration: Duration) -> Timer {
        self.proc.node.set_random_timer(min_duration, max_duration)
    }

    pub fn register_rpc_listener(&self) -> RpcResult<RpcListener> {
        self.proc.node.register_rpc_listener(self.proc.name())
    }

    pub async fn rpc<T: Serialize>(
        &self,
        to: Address,
        tag: u64,
        value: T,
    ) -> RpcResult<RpcResponse> {
        let from = self.proc.address();
        let to = self
            .proc
            .node
            .resolve_addr(&to)
            .ok_or(RpcError::AddressNotResolved)?;
        super::rpc::rpc_impl(from, to, tag, value).await
    }

    pub fn send_local(&self, msg: impl Into<String>) {
        self.proc.send_local_to_user(msg);
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn mount_dir(&self) -> String {
        self.proc.node.mount_dir()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Guard {}

impl Guard {
    pub fn new(ctx: Context) -> Guard {
        CONTEXT.with(|c| assert!(c.borrow().is_none()));
        Context::install(ctx);
        Guard {}
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        Context::reset();
    }
}
