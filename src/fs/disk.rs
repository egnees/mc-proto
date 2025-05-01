use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    event::time::TimeSegment,
    util::trigger::{make_trigger, Trigger, Waiter},
    Address,
};

use super::{
    error::FsError,
    event::{FsEvent, FsEventKind, FsEventOutcome},
    registry::FsEventRegistry,
};

////////////////////////////////////////////////////////////////////////////////

struct Request(Trigger, FsEvent);

////////////////////////////////////////////////////////////////////////////////

pub struct Disk {
    reg: Rc<RefCell<dyn FsEventRegistry>>,
    queue: VecDeque<Request>,
    delay: TimeSegment,
    node: String,
    capacity: usize,
    used: usize,
    in_process: bool,
}

impl Disk {
    pub fn new(
        reg: Rc<RefCell<dyn FsEventRegistry>>,
        delay: TimeSegment,
        node: String,
        capacity: usize,
    ) -> Self {
        Self {
            reg,
            delay,
            node,
            capacity,
            used: 0,
            queue: Default::default(),
            in_process: false,
        }
    }

    pub fn enqueue_request(&mut self, proc: String, kind: FsEventKind) -> Waiter {
        let outcome = if let FsEventKind::Write { len, .. } = kind {
            if self.used + len > self.capacity {
                Err(FsError::StorageLimitReached)
            } else {
                self.used += len;
                Ok(())
            }
        } else {
            Ok(())
        };

        let event = FsEvent {
            delay: self.delay,
            initiated_by: Address::new(self.node.clone(), proc),
            kind,
            outcome,
        };

        self.reg.borrow_mut().register_event_initiated(&event);

        let (waiter, trigger) = make_trigger();
        let request = Request(trigger, event);
        self.queue.push_back(request);

        if !self.in_process {
            self.process_next_request();
        }

        waiter
    }

    pub fn on_request_completed(
        &mut self,
        proc: String,
        kind: FsEventKind,
        outcome: FsEventOutcome,
    ) {
        let event = FsEvent {
            delay: self.delay,
            initiated_by: Address::new(self.node.clone(), proc),
            kind,
            outcome,
        };

        self.reg.borrow_mut().register_event_happen(&event);

        self.in_process = false;

        self.process_next_request();
    }

    fn process_next_request(&mut self) {
        assert!(!self.in_process);
        let Some(request) = self.queue.pop_front() else {
            return;
        };
        self.in_process = true;
        self.reg
            .borrow_mut()
            .register_event_pipelined(request.0, &request.1);
    }

    pub fn file_deleted(&mut self, size: usize) {
        self.used -= size;
    }

    pub fn crash(&mut self) {
        self.in_process = false;
        self.used = 0;
        self.queue.clear();
    }

    pub fn shutdown(&mut self) {
        self.in_process = false;
        self.queue.clear();
    }
}
