use serde::{Deserialize, Serialize};

use crate::addr;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppendEntriesRPC {
    pub term: u64,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    // pub entries: Vec<_>,
    pub leader_commit: u64,
}

impl From<&mc::RpcRequest> for AppendEntriesRPC {
    fn from(value: &mc::RpcRequest) -> Self {
        value.unpack().unwrap()
    }
}

impl AppendEntriesRPC {
    pub const TAG: u64 = 2;

    pub async fn send(&self, to: usize) -> mc::RpcResult<AppendEntriesResult> {
        let to = addr::make_addr(to);
        mc::rpc(to, Self::TAG, self)
            .await
            .map(mc::RpcResponse::into)
    }

    pub fn new(term: u64, prev_log_index: u64, prev_log_term: u64, leader_commit: u64) -> Self {
        Self {
            term,
            prev_log_index,
            prev_log_term,
            leader_commit,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesResult {
    pub term: u64,
    pub success: bool,
}

impl From<mc::RpcResponse> for AppendEntriesResult {
    fn from(value: mc::RpcResponse) -> Self {
        value.unpack().unwrap()
    }
}

impl AppendEntriesResult {
    pub fn new(term: u64, success: bool) -> Self {
        Self { term, success }
    }
}
