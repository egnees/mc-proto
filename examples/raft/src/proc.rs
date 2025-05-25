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

async fn on_init(state: Rc<RefCell<Option<rsm::StateHandle>>>, nodes: usize, me: usize) {
    let s = rsm::StateHandle::new(nodes, me).await;
    let state = state.borrow_mut().insert(s).clone();

    let mut listener = dsbuild::RpcListener::register().unwrap();
    loop {
        let request = listener.listen().await;
        match request.tag() {
            rsm::AppendEntriesRPC::TAG => {
                let res = state
                    .on_append_request(addr::id(request.from()), (&request).into())
                    .await;
                let _ = request.reply(&res);
            }
            rsm::RequestVoteRPC::TAG => {
                let res = state
                    .on_vote_request(addr::id(request.from()), (&request).into())
                    .await;
                let _ = request.reply(&res);
            }
            _ => unreachable!(),
        }
    }
}

impl dsbuild::Process for Raft {
    fn on_message(&mut self, _from: dsbuild::Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let req: req::Request = content.into();
        match req {
            req::Request::Init { nodes, me } => {
                dsbuild::spawn(on_init(self.state.clone(), nodes, me));
            }
            req::Request::Command(cmd) => {
                let resp = self
                    .state
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .on_user_command(cmd);
                if let Some(resp) = resp {
                    dsbuild::send_local(resp);
                }
            }
        };
    }

    fn hash(&self) -> dsbuild::HashType {
        let mut h = DefaultHasher::new();
        match &*self.state.borrow() {
            Some(state) => state.hash(&mut h),
            None => 0.hash(&mut h),
        }
        h.finish()
    }
}
