use std::collections::{HashMap, HashSet, hash_map::Entry};

use raft::{addr::make_addr, proc::Raft, rsm::StateHandle};

////////////////////////////////////////////////////////////////////////////////

pub fn leader(s: mc::SystemHandle, nodes: usize) -> Result<Option<u64>, String> {
    let mut leader = None;
    for n in 0..nodes {
        let proc = make_addr(n);
        let Some(state) = s.proc_state::<Raft>(proc) else {
            continue;
        };
        let cur = state
            .borrow()
            .handle()
            .and_then(|s: StateHandle| s.who_is_leader())
            .ok_or(format!("node {n} not selected leader"))?;
        if cur != *leader.get_or_insert(cur) {
            return Err(format!(
                "node {n} selected leader {cur}, but the rest nodes selected {}",
                leader.unwrap()
            ));
        }
    }
    Ok(leader)
}

pub fn agree_about_leader(s: mc::SystemHandle, nodes: usize) -> Result<(), String> {
    leader(s, nodes).map(|_| ())
}

////////////////////////////////////////////////////////////////////////////////

pub fn some_node_shutdown(s: mc::SystemHandle) -> Result<(), String> {
    if s.stat().nodes_shutdown > 0 {
        Ok(())
    } else {
        Err("no nodes shutdown".into())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn concurrent_candidates_appear_count(
    s: mc::SystemHandle,
    max_candidates: usize,
    window: u64,
) -> usize {
    let mut cnt = 0;
    let mut candidate_timers: HashSet<usize> = HashSet::default();
    let mut counts: HashMap<u64, usize> = HashMap::default();
    for e in s.log().iter() {
        match e {
            mc::LogEntry::TimerFired(timer) => {
                if candidate_timers.get(&timer.id).is_some() {
                    let time = {
                        let mut x = timer.time.as_millis() as u64;
                        x -= x % window;
                        x
                    };
                    let entry = counts.entry(time);
                    let cur_cnt = match entry {
                        Entry::Occupied(mut e) => {
                            *e.get_mut() += 1;
                            *e.get()
                        }
                        Entry::Vacant(e) => {
                            e.insert(1);
                            1
                        }
                    };
                    if cur_cnt == max_candidates {
                        cnt += 1;
                    }
                }
            }
            mc::LogEntry::TimerSet(timer) => {
                if timer.min_duration.as_millis() == 250 && timer.max_duration.as_millis() == 750 {
                    let result = candidate_timers.insert(timer.id);
                    assert!(result);
                }
            }
            _ => {}
        }
    }
    cnt
}

////////////////////////////////////////////////////////////////////////////////

pub fn no_two_leaders_in_one_term(s: mc::SystemHandle, nodes: usize) -> Result<(), String> {
    let mut terms = HashSet::new();
    for n in 0..nodes {
        let Some(state) = s.proc_state::<Raft>(make_addr(n)) else {
            continue;
        };
        let Some(state) = state.borrow().handle() else {
            continue;
        };
        let term = state.current_term();
        if state.is_leader() {
            let first_one = terms.insert(term);
            if !first_one {
                return Err("two leaders in one term".into());
            }
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

pub fn restrict_depth(s: mc::StateView, max_depth: usize) -> Result<(), String> {
    if s.depth() > max_depth {
        Err(format!("too big depth: {}", s.depth()))
    } else {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn raft_invariants(s: mc::StateView, nodes: usize, max_depth: usize) -> Result<(), String> {
    restrict_depth(s.clone(), max_depth)?;
    no_two_leaders_in_one_term(s.system(), nodes)
}
