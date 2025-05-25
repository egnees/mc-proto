use crate::{model::context::Context, Address};

use super::{error::TcpError, stream::TcpStream};

////////////////////////////////////////////////////////////////////////////////

pub struct TcpListener;

impl TcpListener {
    pub async fn listen() -> Result<TcpStream, TcpError> {
        let context = Context::current();
        let reg = context.event_manager.tcp_registry();
        let me = context.proc.address();
        TcpStream::listen(me, reg).await
    }

    pub async fn listen_to(to: &Address) -> Result<TcpStream, TcpError> {
        let context = Context::current();
        let reg = context.event_manager.tcp_registry();
        let me = context.proc.address();
        TcpStream::listen_to(me, to.clone(), reg).await
    }
}
