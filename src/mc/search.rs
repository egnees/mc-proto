use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use super::{
    log::Log,
    process::CreateProcessFn,
    state::{State, System},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Clone)]
struct SearchVertex {
    trace: Vec<(usize, bool)>,
}

#[derive(Default)]
pub struct Searcher {
    create: Vec<CreateProcessFn>,
}

impl Searcher {
    pub fn add_process(&mut self, f: CreateProcessFn) {
        self.create.push(f);
    }

    fn system_from_vertex(
        &self,
        init: &mut dyn FnMut(&mut System),
        vertex: &SearchVertex,
    ) -> System {
        System::from_trace_and_proc(&vertex.trace, &self.create, init)
    }

    pub fn make_search(
        &self,
        depth: usize,
        mut init: impl FnMut(&mut System),
        prune: impl Fn(Rc<RefCell<State>>) -> bool,
        check: impl Fn(Rc<RefCell<State>>) -> bool,
    ) -> Option<Log> {
        let mut q: VecDeque<SearchVertex> = VecDeque::new();
        q.push_back(SearchVertex::default());
        let mut cnt = 0;
        while let Some(vertex) = q.pop_front() {
            cnt += 1;
            let sys = self.system_from_vertex(&mut init, &vertex);
            let state = sys.state();
            if prune(state.clone()) {
                continue;
            }
            if sys.pending_events_count() == 0 || vertex.trace.len() == depth {
                let check_result = check(state.clone());
                if !check_result {
                    println!("CNT = {}", cnt);
                    return Some(state.borrow().log.clone());
                }
            }
            let pending_events = sys.pending_events_count();
            for i in 0..pending_events {
                {
                    let mut v1 = vertex.clone();
                    v1.trace.push((i, false));
                    q.push_back(v1);
                }
                {
                    let mut v1 = vertex.clone();
                    v1.trace.push((i, true));
                    q.push_back(v1);
                }
            }
        }
        None
    }
}
