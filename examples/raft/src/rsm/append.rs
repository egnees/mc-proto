use serde::{Deserialize, Serialize};

use crate::addr;

use super::log::LogEntry;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppendEntriesRPC {
    pub term: u64,
    pub prev_log_index: usize,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: usize,
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

    pub fn new_hb(
        term: u64,
        prev_log_index: usize,
        prev_log_term: u64,
        leader_commit: usize,
    ) -> Self {
        Self {
            term,
            prev_log_index,
            prev_log_term,
            entries: Vec::default(),
            leader_commit,
        }
    }

    pub fn new(
        term: u64,
        prev_log_index: usize,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: usize,
    ) -> Self {
        Self {
            term,
            prev_log_index,
            prev_log_term,
            entries,
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
