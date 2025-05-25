use std::{cell::RefCell, hash::Hash, rc::Rc};

use dsbuild::JoinHandle;

use crate::{
    cmd::{Command, Error, Response},
    db::DataBase,
};

use super::{
    append::{AppendEntriesRPC, AppendEntriesResult},
    election::{election_timeout, make_election_timeout},
    heartbeat::send_heartbeats,
    log::{Log, LogEntry, replicate_log_with_result},
    term::Term,
    vote::{RequestVoteRPC, RequestVoteResult, VotedFor},
};

////////////////////////////////////////////////////////////////////////////////

pub struct Common {
    current_term: Term,
    voted_for: VotedFor,
    commit_index: usize,
    last_applied: usize,
    nodes: usize,
    me: usize,
    db: DataBase,
    log: Log,
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
            db: DataBase::default(),
            log: Log::new().await,
        }
    }

    fn apply_log(&mut self) {
        while self.commit_index > self.last_applied {
            self.last_applied += 1;
            let cmd = &self.log.entry(self.last_applied).cmd;
            let resp = self.db.apply(&cmd.kind);
            let resp = cmd.response(resp);
            if cmd.leader == self.me {
                dsbuild::send_local(resp);
            }
        }
    }
}

impl Hash for Common {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.current_term.get().hash(state);
        self.voted_for.get().hash(state);
        self.commit_index.hash(state);
        self.last_applied.hash(state);
        self.log.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Leader {
    next_index: Vec<usize>,
    match_index: Vec<usize>,
    heartbeat: JoinHandle<()>,
    replication: JoinHandle<()>,
}

impl Leader {
    pub fn new(
        nodes: usize,
        last_log_index: usize,
        heartbeat: JoinHandle<()>,
        replication: JoinHandle<()>,
    ) -> Self {
        Self {
            next_index: vec![last_log_index + 1; nodes],
            match_index: vec![0; nodes],
            heartbeat,
            replication,
        }
    }

    pub fn commit_index(
        &mut self,
        mut prev_index: usize,
        log: &Log,
        term: u64,
        need: usize,
    ) -> usize {
        while prev_index < log.last_log_index() {
            let idx = prev_index + 1;
            if log.entry(idx).term != term {
                break;
            }
            let match_cnt = self.match_index.iter().filter(|i| **i >= idx).count();
            if match_cnt >= need {
                prev_index += 1;
            } else {
                break;
            }
        }
        prev_index
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
        election: JoinHandle<()>,
        current_leader: Option<u64>,
    },
    Candidate {
        election: JoinHandle<()>,
    },
    Leader(Leader),
}

impl Role {
    fn cancel_async_tasks(&mut self) {
        match self {
            Role::Idle => {}
            Role::Follower { election, .. } => election.abort(),
            Role::Candidate { election } => election.abort(),
            Role::Leader(leader) => {
                leader.heartbeat.abort();
                leader.replication.abort();
            }
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

    ////////////////////////////////////////////////////////////////////////////////

    pub fn log(&self) -> Vec<LogEntry> {
        self.inner.borrow().common.log.clone().into()
    }

    ////////////////////////////////////////////////////////////////////////////////

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
            self.transit_to_follower(req.term, Some(leader_id), Some(leader_id))
                .await;
        }

        let current_term = self.current_term();
        if req.term < current_term {
            return AppendEntriesResult {
                term: current_term,
                success: false,
            };
        }

        let handle = self.inner.borrow_mut().common.log.append_from_leader(
            req.prev_log_index,
            req.prev_log_term,
            req.entries,
        );

        let Some(handle) = handle else {
            return AppendEntriesResult {
                term: current_term,
                success: false,
            };
        };

        handle.await.unwrap();

        let mut state = self.inner.borrow_mut();
        state.common.commit_index = state
            .common
            .commit_index
            .max(req.leader_commit)
            .min(state.common.log.last_log_index());
        state.common.apply_log();

        AppendEntriesResult {
            term: current_term,
            success: true,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn on_user_command(&self, cmd: Command) -> Option<Response> {
        let leader = self.who_is_leader().map(|leader| leader as usize);
        let mut state = self.inner.borrow_mut();
        let term = state.common.current_term.get();
        if !matches!(*state.role.borrow(), Role::Leader(_)) {
            return Some(Response::new_error(
                cmd.id,
                Error::NotLeader {
                    redirected_to: leader,
                },
            ));
        }
        let handle = state.common.log.append_from_user(LogEntry { term, cmd });
        dsbuild::spawn({
            let state = self.clone();
            async move {
                handle.await.unwrap();
                let mut replication = dsbuild::spawn(replicate_log_with_result(state.clone()));
                {
                    let state = state.inner.borrow();
                    if let Role::Leader(leader) = &mut *state.role.borrow_mut() {
                        std::mem::swap(&mut leader.replication, &mut replication);
                        replication.abort();
                    }
                }
            }
        });
        None
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Transit from leader or candidate to follower
    /// when request with greater term received
    pub async fn transit_to_follower(
        &self,
        new_term: u64,
        new_leader: Option<u64>,
        new_vote_for: Option<u64>,
    ) {
        dsbuild::log("transit to follower");

        let vf = self.set_voted_for(new_vote_for);
        let term = self.set_current_term(new_term);

        let election = dsbuild::spawn({
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
        dsbuild::log("transit to candidate");

        let nodes = self.nodes();
        let me = self.me();

        let vf = self.set_voted_for(Some(me as u64));

        let election = dsbuild::spawn({
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
        dsbuild::log("transit to leader");
        let hb = dsbuild::spawn(send_heartbeats(self.clone()));
        let replication = dsbuild::spawn(replicate_log_with_result(self.clone()));
        let leader = Leader::new(
            self.nodes(),
            self.inner.borrow().common.log.last_log_index(),
            hb,
            replication,
        );
        self.set_role(Role::Leader(leader));
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn make_request_vote_rpc(&self) -> RequestVoteRPC {
        let state = self.inner.borrow();
        RequestVoteRPC::new(state.common.current_term.get(), 0, 0)
    }

    pub fn make_heartbeat(&self) -> AppendEntriesRPC {
        let state = self.inner.borrow();
        AppendEntriesRPC::new_hb(
            state.common.current_term.get(),
            0,
            0,
            state.common.commit_index,
        )
    }

    fn increment_term(&self) -> (u64, JoinHandle<()>) {
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

    pub fn set_current_term(&self, new_term: u64) -> dsbuild::JoinHandle<()> {
        self.inner.borrow().common.current_term.set(new_term)
    }

    pub fn who_is_leader(&self) -> Option<u64> {
        let state = self.inner.borrow();
        match &*state.role.borrow() {
            Role::Idle => None,
            Role::Follower { current_leader, .. } => *current_leader,
            Role::Candidate { .. } => None,
            Role::Leader(_) => Some(state.common.me as u64),
        }
    }

    pub fn is_candidate(&self) -> bool {
        let state = self.inner.borrow();
        matches!(&*state.role.borrow(), Role::Candidate { .. })
    }

    pub fn is_leader(&self) -> bool {
        let state = self.inner.borrow();
        matches!(&*state.role.borrow(), Role::Leader { .. })
    }

    pub fn set_voted_for(&self, voted_for: Option<u64>) -> dsbuild::JoinHandle<()> {
        self.inner.borrow().common.voted_for.set(voted_for)
    }

    pub fn voted_for(&self) -> Option<u64> {
        self.inner.borrow().common.voted_for.get()
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn make_append_request_for_follower(&self, i: usize) -> Option<AppendEntriesRPC> {
        let state = self.inner.borrow();
        let last_log_index = state.common.log.last_log_index();
        match &*state.role.borrow() {
            Role::Leader(Leader { next_index, .. }) => {
                let idx = next_index[i];
                if last_log_index >= idx {
                    let entries = state.common.log.entries(idx, last_log_index).to_vec();
                    Some(AppendEntriesRPC {
                        term: state.common.current_term.get(),
                        prev_log_index: idx - 1,
                        prev_log_term: state.common.log.log_term(idx - 1),
                        entries,
                        leader_commit: state.common.commit_index,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn dec_next_index(&self, i: usize) {
        let state = self.inner.borrow();
        if let Role::Leader(leader) = &mut *state.role.borrow_mut() {
            leader.next_index[i] -= 1;
        }
    }

    pub fn upd_follower_info(&self, i: usize, next_index: usize, match_index: usize) {
        let state = self.inner.borrow();
        if let Role::Leader(leader) = &mut *state.role.borrow_mut() {
            leader.next_index[i] = next_index;
            leader.match_index[i] = match_index;
        }
    }

    pub fn last_log_index(&self) -> usize {
        self.inner.borrow().common.log.last_log_index()
    }

    pub fn upd_commit_index_and_apply_log(&self) -> bool {
        let index = {
            let state = self.inner.borrow();
            match &mut *state.role.borrow_mut() {
                Role::Leader(leader) => {
                    let commit_index = leader.commit_index(
                        state.common.commit_index,
                        &state.common.log,
                        state.common.current_term.get(),
                        state.common.nodes / 2 + 1,
                    );
                    assert!(commit_index >= state.common.commit_index);
                    commit_index
                }
                _ => state.common.commit_index,
            }
        };
        let mut state = self.inner.borrow_mut();
        if state.common.commit_index < index {
            state.common.commit_index = index;
            state.common.apply_log();
            true
        } else {
            false
        }
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
