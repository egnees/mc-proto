use serde::{Deserialize, Serialize};

use super::{addr, replicated::RepliactedU64};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct RequestVoteRPC {
    /// candidates term
    term: u64,

    candidate_id: u64,

    last_log_index: u64,
    last_log_term: u64,
}

impl From<&mc::RpcRequest> for RequestVoteRPC {
    fn from(value: &mc::RpcRequest) -> Self {
        value.unpack().unwrap()
    }
}

impl RequestVoteRPC {
    const TAG: u64 = 1;

    pub async fn send(&self, to: u64) -> mc::RpcResult<RequestVoteResult> {
        let to = addr::make_addr(to);
        mc::rpc(to, Self::TAG, self)
            .await
            .map(mc::RpcResponse::into)
    }

    pub fn new(term: u64, candidate_id: u64, last_log_index: u64, last_log_term: u64) -> Self {
        Self {
            term,
            candidate_id,
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
    term: u64,
    vote_granted: bool,
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

#[derive(Default)]
pub struct VotedFor {
    value: RepliactedU64,
}

impl VotedFor {
    pub async fn get(&mut self) -> mc::FsResult<Option<u64>> {
        self.value
            .read()
            .await
            .map(|v| if v == 0 { None } else { Some(v - 1) })
    }

    pub async fn set(&mut self, value: Option<u64>) -> mc::FsResult<()> {
        self.value.update(value.map(|v| v + 1).unwrap_or(0)).await
    }
}
