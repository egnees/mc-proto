use std::{cell::RefCell, hash::Hash, rc::Rc};

use mc::JoinHandle;

use super::{
    append::{AppendEntriesRPC, AppendEntriesResult},
    election::{election_timeout, make_election_timeout},
    heartbeat::send_heartbeats,
    term::Term,
    vote::{RequestVoteRPC, RequestVoteResult, VotedFor},
};

////////////////////////////////////////////////////////////////////////////////

pub struct Common {
    current_term: Term,
    voted_for: VotedFor,
    commit_index: u64,
    last_applied: u64,
    nodes: usize,
    me: usize,
}

impl Common {
    pub async fn new(nodes: usize, me: usize) -> Self {
        Self {
            current_term: Term::new().await,
            voted_for: VotedFor::new().await,
            commit_index: 0,
            last_applied: 0,
            nodes,
            me,
        }
    }
}

impl Hash for Common {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.current_term.get().hash(state);
        self.voted_for.get().hash(state);
        self.commit_index.hash(state);
        self.last_applied.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Leader {
    next_index: Vec<u64>,
    match_index: Vec<u64>,
    heartbeat: JoinHandle<()>,
}

impl Leader {
    pub fn new(nodes: usize, last_log_index: u64, heartbeat: JoinHandle<()>) -> Self {
        Self {
            next_index: vec![last_log_index + 1; nodes],
            match_index: vec![0; nodes],
            heartbeat,
        }
    }
}

impl Hash for Leader {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.next_index.hash(state);
        self.match_index.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum Role {
    Idle,
    Follower {
        election: mc::JoinHandle<()>,
        current_leader: Option<u64>,
    },
    Candidate {
        election: mc::JoinHandle<()>,
    },
    Leader(Leader),
}

impl Role {
    fn cancel_async_tasks(&mut self) {
        match self {
            Role::Idle => {}
            Role::Follower { election, .. } => election.abort(),
            Role::Candidate { election } => election.abort(),
            Role::Leader(leader) => leader.heartbeat.abort(),
        }
    }
}

impl Drop for Role {
    fn drop(&mut self) {
        self.cancel_async_tasks();
    }
}

impl Hash for Role {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Role::Idle => 0.hash(state),
            Role::Follower { current_leader, .. } => {
                1.hash(state);
                current_leader.hash(state)
            }
            Role::Candidate { .. } => 2.hash(state),
            Role::Leader(leader) => {
                3.hash(state);
                leader.hash(state);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct State {
    common: Common,
    role: Rc<RefCell<Role>>,
}

impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.common.hash(state);
        self.role.borrow().hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StateHandle {
    inner: Rc<RefCell<State>>,
}

impl StateHandle {
    pub async fn new(nodes: usize, me: usize) -> Self {
        let common = Common::new(nodes, me).await;
        let term = common.current_term.get();
        let voted_for = common.voted_for.get();
        let state = State {
            common,
            role: Rc::new(RefCell::new(Role::Idle)),
        };
        let handle = Self {
            inner: Rc::new(RefCell::new(state)),
        };
        handle.transit_to_follower(term, None, voted_for).await;
        handle
    }

    pub async fn on_vote_request(
        &self,
        candidate_id: u64,
        req: RequestVoteRPC,
    ) -> RequestVoteResult {
        if self.current_term() < req.term {
            self.transit_to_follower(req.term, None, Some(candidate_id))
                .await;
        }
        let current_term = self.current_term();
        if req.term < current_term {
            return RequestVoteResult {
                term: current_term,
                vote_granted: false,
            };
        }
        let vote_granted = if self.voted_for().unwrap_or(candidate_id) == candidate_id {
            self.set_voted_for(Some(candidate_id)).await.unwrap();
            true
        } else {
            false
        };
        RequestVoteResult {
            term: current_term,
            vote_granted,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub async fn on_append_request(
        &self,
        leader_id: u64,
        req: AppendEntriesRPC,
    ) -> AppendEntriesResult {
        if self.current_term() <= req.term {
            self.transit_to_follower(req.term, Some(leader_id as u64), Some(leader_id as u64))
                .await;
        }

        let current_term = self.current_term();
        let success = if req.term < current_term { false } else { true };
        AppendEntriesResult {
            term: current_term,
            success,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Transit from leader or candidate to follower
    /// when request with greater term received
    async fn transit_to_follower(
        &self,
        new_term: u64,
        new_leader: Option<u64>,
        new_vote_for: Option<u64>,
    ) {
        mc::log("transit to follower");

        let vf = self.set_voted_for(new_vote_for);
        let term = self.set_current_term(new_term);

        let election = mc::spawn({
            let state = self.clone();
            async move {
                let _ = election_timeout().await;
                state.transit_to_candidate();
            }
        });

        self.set_role(Role::Follower {
            election,
            current_leader: new_leader,
        });

        let _ = tokio::join!(vf, term);
    }

    /// Transit from follower to candidate on election timeout
    fn transit_to_candidate(&self) {
        mc::log("transit to candidate");

        let nodes = self.nodes();
        let me = self.me();

        let vf = self.set_voted_for(Some(me as u64));

        let election = mc::spawn({
            let state = self.clone();
            async move {
                vf.await.unwrap();
                loop {
                    let (_, term) = state.increment_term();
                    term.await.unwrap();

                    let rpc = state.make_request_vote_rpc();
                    let result = make_election_timeout(nodes, me, rpc).await;

                    match result {
                        Ok(_) => {
                            state.transit_to_leader();
                            break;
                        }
                        Err(Some(new_term)) => {
                            state.transit_to_follower(new_term, None, None).await;
                            break;
                        }
                        Err(None) => {
                            continue;
                        }
                    }
                }
            }
        });

        self.set_role(Role::Candidate { election });
    }

    /// Transit from candidate to leader on majority of votes received
    fn transit_to_leader(&self) {
        mc::log("transit to leader");
        let hb = mc::spawn(send_heartbeats(self.clone()));
        let leader = Leader::new(self.nodes(), 0, hb);
        self.set_role(Role::Leader(leader));
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn make_request_vote_rpc(&self) -> RequestVoteRPC {
        let state = self.inner.borrow();
        RequestVoteRPC::new(state.common.current_term.get(), 0, 0)
    }

    pub fn make_heartbeat(&self) -> AppendEntriesRPC {
        let state = self.inner.borrow();
        AppendEntriesRPC::new(
            state.common.current_term.get(),
            0,
            0,
            state.common.commit_index,
        )
    }

    fn increment_term(&self) -> (u64, mc::JoinHandle<()>) {
        self.inner.borrow().common.current_term.increment()
    }

    pub fn nodes(&self) -> usize {
        self.inner.borrow().common.nodes
    }

    pub fn me(&self) -> usize {
        self.inner.borrow().common.me
    }

    pub fn current_term(&self) -> u64 {
        self.inner.borrow().common.current_term.get()
    }

    pub fn set_current_term(&self, new_term: u64) -> mc::JoinHandle<()> {
        self.inner.borrow().common.current_term.set(new_term)
    }

    pub fn who_is_leader(&self) -> Option<u64> {
        let state = self.inner.borrow();
        match &*state.role.borrow() {
            Role::Idle => None,
            Role::Follower { current_leader, .. } => current_leader.clone(),
            Role::Candidate { .. } => None,
            Role::Leader(_) => Some(state.common.me as u64),
        }
    }

    pub fn is_candidate(&self) -> bool {
        let state = self.inner.borrow();
        match &*state.role.borrow() {
            Role::Candidate { .. } => true,
            _ => false,
        }
    }

    pub fn is_leader(&self) -> bool {
        let state = self.inner.borrow();
        match &*state.role.borrow() {
            Role::Leader { .. } => true,
            _ => false,
        }
    }

    pub fn set_voted_for(&self, voted_for: Option<u64>) -> mc::JoinHandle<()> {
        self.inner.borrow().common.voted_for.set(voted_for)
    }

    pub fn voted_for(&self) -> Option<u64> {
        self.inner.borrow().common.voted_for.get()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn set_role(&self, role: Role) {
        *self.inner.borrow().role.borrow_mut() = role;
    }
}

impl Hash for StateHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.borrow().hash(state);
    }
}
