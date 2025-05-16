use std::{cell::RefCell, rc::Rc, time::Duration};

use mc::oneshot::Sender;

use crate::addr;

use super::vote::RequestVoteRPC;

////////////////////////////////////////////////////////////////////////////////

pub struct Counter {
    sender: Option<Sender<Result<(), u64>>>,
    count: usize,
    need: usize,
}

impl Counter {
    fn new(sender: Sender<Result<(), u64>>, need: usize) -> Self {
        Self {
            sender: Some(sender),
            count: 0,
            need,
        }
    }

    fn inc(&mut self) {
        self.count += 1;
        if self.count == self.need {
            self.sender.take().unwrap().send(Ok(())).unwrap();
        }
    }

    fn err(&mut self, term: u64) {
        self.sender.take().unwrap().send(Err(term)).unwrap();
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn election_timeout() -> mc::Timer {
    // from raft article
    let min_duration = Duration::from_millis(250);
    let max_duration = Duration::from_millis(750);
    mc::set_random_timer(min_duration, max_duration)
}

////////////////////////////////////////////////////////////////////////////////

async fn make_election(nodes: usize, me: usize, r: RequestVoteRPC) -> Result<(), u64> {
    let term = r.term;
    let (sender, recv) = mc::oneshot::channel();
    let counter = Counter::new(sender, nodes / 2 + 1);
    let counter = Rc::new(RefCell::new(counter));
    let handles = addr::iter_others(nodes, me).map(|n| {
        mc::spawn({
            let r = r.clone();
            let counter = counter.clone();
            async move {
                let result = r.send(n).await;
                if let Ok(result) = result {
                    if result.vote_granted {
                        counter.borrow_mut().inc();
                    } else if term < result.term {
                        counter.borrow_mut().err(result.term);
                    }
                }
            }
        })
    });
    let _s = mc::CancelSet::from_iter(handles);
    counter.borrow_mut().inc(); // for myself
    recv.await.unwrap()
}

////////////////////////////////////////////////////////////////////////////////

pub async fn make_election_timeout(
    nodes: usize,
    me: usize,
    rpc: RequestVoteRPC,
) -> Result<(), Option<u64>> {
    tokio::select! {
        _ = election_timeout() => {
            Err(None)
        }
        r = make_election(nodes, me, rpc.clone()) => {
            r.map_err(|t| Some(t))
        }
    }
}
