use crate::{model::context::Context, Address};

use super::{error::TcpError, stream::TcpStream};

////////////////////////////////////////////////////////////////////////////////

/// Allows process to listen to TCP connections.
pub struct TcpListener;

impl TcpListener {
    /// Returns after TCP connection is established.
    pub async fn listen() -> Result<TcpStream, TcpError> {
        let context = Context::current();
        let reg = context.event_manager.tcp_registry();
        let me = context.proc.address();
        TcpStream::listen(me, reg).await
    }

    /// Allows to listen to the connections from the specified address.
    pub async fn listen_to(to: &Address) -> Result<TcpStream, TcpError> {
        let context = Context::current();
        let reg = context.event_manager.tcp_registry();
        let me = context.proc.address();
        TcpStream::listen_to(me, to.clone(), reg).await
    }
}
