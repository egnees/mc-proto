use serde::{Deserialize, Serialize};

use super::replicated::RepliactedU64;
use crate::addr;

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub struct RequestVoteRPC {
    // candidates term
    pub term: u64,

    // candidate id = RpcRequest::from
    // pub candidate_id: usize,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

impl From<&mc::RpcRequest> for RequestVoteRPC {
    fn from(value: &mc::RpcRequest) -> Self {
        value.unpack().unwrap()
    }
}

impl RequestVoteRPC {
    pub const TAG: u64 = 1;

    pub async fn send(&self, to: usize) -> mc::RpcResult<RequestVoteResult> {
        let to = addr::make_addr(to);
        mc::rpc(to, Self::TAG, self)
            .await
            .map(mc::RpcResponse::into)
    }

    pub fn new(term: u64, last_log_index: u64, last_log_term: u64) -> Self {
        Self {
            term,
            last_log_index,
            last_log_term,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct RequestVoteResult {
    /// current term of receiver,
    /// for candidate to update itself
    pub term: u64,
    pub vote_granted: bool,
}

impl From<mc::RpcResponse> for RequestVoteResult {
    fn from(value: mc::RpcResponse) -> Self {
        value.unpack().unwrap()
    }
}

impl RequestVoteResult {
    pub fn new(term: u64, vote_granted: bool) -> Self {
        Self { term, vote_granted }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct VotedFor {
    value: RepliactedU64,
}

impl VotedFor {
    pub async fn new() -> Self {
        Self {
            value: RepliactedU64::new("vote.txt").await,
        }
    }

    pub fn get(&self) -> Option<u64> {
        let v = self.value.read();
        if v == 0 { None } else { Some(v - 1) }
    }

    pub fn set(&self, value: Option<u64>) -> mc::JoinHandle<()> {
        self.value.update(value.map(|v| v + 1).unwrap_or(0))
    }
}
