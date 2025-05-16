use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use crate::{addr, req, rsm};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Raft {
    state: Rc<RefCell<Option<rsm::StateHandle>>>,
}

impl Raft {
    pub fn handle(&self) -> Option<rsm::StateHandle> {
        self.state.borrow().clone()
    }
}

impl mc::Process for Raft {
    fn on_message(&mut self, _from: mc::Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let req: req::Request = content.into();
        match req {
            req::Request::Init { nodes, me } => mc::spawn({
                let state = self.state.clone();
                async move {
                    let s = rsm::StateHandle::new(nodes, me).await;
                    let state = state.borrow_mut().insert(s).clone();

                    let mut listener = mc::RpcListener::register().unwrap();
                    mc::spawn(async move {
                        loop {
                            let request = listener.listen().await;
                            match request.tag {
                                rsm::AppendEntriesRPC::TAG => {
                                    let res = state
                                        .on_append_request(
                                            addr::id(request.from()),
                                            (&request).into(),
                                        )
                                        .await;
                                    let _ = request.reply(&res);
                                }
                                rsm::RequestVoteRPC::TAG => {
                                    let res = state
                                        .on_vote_request(
                                            addr::id(request.from()),
                                            (&request).into(),
                                        )
                                        .await;
                                    let _ = request.reply(&res);
                                }
                                _ => unreachable!(),
                            }
                        }
                    });
                }
            }),
        };
    }

    fn hash(&self) -> mc::HashType {
        let mut h = DefaultHasher::new();
        match &*self.state.borrow() {
            Some(state) => state.hash(&mut h),
            None => 0.hash(&mut h),
        }
        h.finish()
    }
}
