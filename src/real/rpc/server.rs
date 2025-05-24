use std::net::SocketAddr;

use real_rpc::rpc_server::{Rpc, RpcServer};
use real_rpc::{RpcRequest, RpcResponse};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use super::request;

////////////////////////////////////////////////////////////////////////////////

pub mod real_rpc {
    tonic::include_proto!("real_rpc");
}

////////////////////////////////////////////////////////////////////////////////

pub struct RealRpcServer {
    sender: UnboundedSender<request::RpcRequest>,
}

#[tonic::async_trait]
impl Rpc for RealRpcServer {
    async fn send(&self, request: Request<RpcRequest>) -> Result<Response<RpcResponse>, Status> {
        let request = request.into_inner();
        let (sender, receiver) = oneshot::channel();
        let request = request::RpcRequest {
            from: request.from.into(),
            tag: request.tag,
            content: request.content,
            resp: sender,
        };
        let send_result = self.sender.send(request);
        if send_result.is_err() {
            return Err(Status::unknown("the request was not received"));
        }
        let result = receiver.await;
        match result {
            Ok(content) => {
                let r = real_rpc::RpcResponse { content };
                Ok(Response::new(r))
            }
            Err(_) => Err(Status::unknown("the request was not replied")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn listen(addr: SocketAddr, sender: UnboundedSender<request::RpcRequest>) {
    let server = RealRpcServer { sender };
    Server::builder()
        .add_service(RpcServer::new(server))
        .serve(addr)
        .await
        .unwrap();
}
