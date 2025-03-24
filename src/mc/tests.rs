use std::{
    cell::{Ref, RefCell},
    collections::BTreeSet,
    rc::Rc,
};

use super::{
    message::Message,
    process::{send_local, send_message, sleep, spawn, Process, ProcessId},
    search::Searcher,
    state::{State, System},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct Ping {
    waiting: Rc<RefCell<i64>>,
}

impl Process for Ping {
    fn on_message(&mut self, _from: ProcessId, _message: Message) {
        *self.waiting.borrow_mut() -= 1;
    }

    fn on_local_message(&mut self, message: Message) {
        *self.waiting.borrow_mut() += 1;
        let x = *self.waiting.borrow();
        let waiting = self.waiting.clone();
        spawn(async move {
            while *waiting.borrow() == x {
                send_message(1, message.clone());
                let _ = sleep(0.5).await;
            }
        });
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct Pong {}

impl Process for Pong {
    fn on_message(&mut self, from: ProcessId, message: Message) {
        send_local(message.clone());
        send_message(from, message);
    }

    fn on_local_message(&mut self, _message: Message) {
        unreachable!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct CorrectPing {
    waiting: Rc<RefCell<BTreeSet<Message>>>,
}

impl Process for CorrectPing {
    fn on_message(&mut self, _: ProcessId, msg: Message) {
        self.waiting.borrow_mut().remove(&msg);
    }

    fn on_local_message(&mut self, message: Message) {
        self.waiting.borrow_mut().insert(message.clone());
        let waiting = self.waiting.clone();
        spawn(async move {
            while waiting.borrow().contains(&message) {
                send_message(1, message.clone());
                let _ = sleep(0.5).await;
            }
        });
    }
}

#[test]
fn simple() {
    let mut s = Searcher::default();
    s.add_process(Box::new(|| Box::new(Ping::default())));
    s.add_process(Box::new(|| Box::new(Pong::default())));
    let init = |system: &mut System| {
        system.send_local(0, "1".into());
        system.send_local(0, "2".into());
    };
    let prune = |state: Rc<RefCell<State>>| {
        let state = state.borrow();
        state.messages_dropped >= 2 || state.timers_fired >= 4
    };
    let check = |state: Rc<RefCell<State>>| {
        let state = state.borrow();
        let msgs = state.process_infos[1]
            .pending_local
            .iter()
            .collect::<BTreeSet<_>>();
        msgs.len() == 2
    };
    let result = s.make_search(7, init, prune, check);
    println!("{}", result.unwrap());
}

#[test]
fn simple_correct() {
    let mut s = Searcher::default();
    s.add_process(Box::new(|| Box::new(CorrectPing::default())));
    s.add_process(Box::new(|| Box::new(Pong::default())));
    let init = |system: &mut System| {
        system.send_local(0, "1".into());
        system.send_local(0, "2".into());
    };
    let prune = |state: Rc<RefCell<State>>| {
        let state = state.borrow();
        state.messages_dropped >= 2 || state.timers_fired >= 2
    };
    let check = |state: Rc<RefCell<State>>| {
        let state = state.borrow();
        let msgs = state.process_infos[1]
            .pending_local
            .iter()
            .collect::<BTreeSet<_>>();
        msgs.len() == 2
    };
    let result = s.make_search(8, init, prune, check);
    // println!("{}", result.unwrap());
    assert!(result.is_none());
}
