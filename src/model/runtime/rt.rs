use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, VecDeque},
    future::Future,
    rc::{Rc, Weak},
    sync::Arc,
};

use crate::{model::proc::ProcessHandle, util};

use super::{
    task::{JoinHandle, Task, TaskId},
    waker::Waker,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct RuntimeState {
    pending: VecDeque<TaskId>,
    tasks: HashMap<TaskId, Task>,
    next_task_id: TaskId,
}

impl RuntimeState {
    fn clear_pending(&mut self) {
        while let Some(h) = self.pending.pop_front() {
            if self.tasks.contains_key(&h) {
                self.pending.push_front(h);
                break;
            }
        }
    }

    fn next_task_owner(&mut self) -> Option<ProcessHandle> {
        self.clear_pending();
        self.pending
            .front()
            .and_then(|t| self.tasks.get(t))
            .map(|t| t.owner.clone())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RuntimeHandle(Weak<RefCell<RuntimeState>>);

////////////////////////////////////////////////////////////////////////////////

impl RuntimeHandle {
    fn is_runtime_dropped(&self) -> bool {
        self.0.strong_count() == 0
    }

    fn state(&self) -> Rc<RefCell<RuntimeState>> {
        self.0.upgrade().unwrap()
    }

    pub fn schedule(&self, task: TaskId) {
        if !self.is_runtime_dropped() {
            self.state().borrow_mut().pending.push_back(task);
        }
    }

    pub fn spawn<F>(&self, task: F, owner: ProcessHandle) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        let binding = self.state();
        let mut state = binding.borrow_mut();

        let task_id = state.next_task_id;
        state.next_task_id += 1;

        let (sender, receiver) = util::oneshot::channel();
        let task = async move {
            let result = task.await;
            let _ = sender.send(result); // receiver can be dropped which is ok
        };
        state.tasks.insert(
            task_id,
            Task {
                future: Box::pin(task),
                owner,
            },
        );
        state.pending.push_back(task_id);

        JoinHandle::new(task_id, receiver, self.clone())
    }

    pub fn cancel_task(&self, task_id: TaskId) {
        if self.is_runtime_dropped() {
            return;
        }
        let task = self.state().borrow_mut().tasks.remove(&task_id);
        let is_some = task.is_some();
        if is_some {
            drop(task);

            let state = self.state();
            let mut state = state.borrow_mut();

            // remove tasks from pending
            let mut v = Default::default();
            std::mem::swap(&mut v, &mut state.pending);
            state.pending = v.into_iter().filter(|id| *id != task_id).collect();
        }
    }

    pub fn cancel_tasks(&self, pred: impl Fn(&ProcessHandle) -> bool) {
        if self.is_runtime_dropped() {
            return;
        }
        loop {
            let to_cancel = {
                let state = self.state();
                let state = state.borrow();
                state
                    .tasks
                    .iter()
                    .filter(|(_id, t)| pred(&t.owner))
                    .map(|(id, _)| *id)
                    .collect::<BTreeSet<_>>()
            };

            if to_cancel.is_empty() {
                break;
            }

            to_cancel.iter().for_each(|task| {
                // task will be dropped after state borrow is released
                // which is important, because task drop can lead
                // to scheduling of another tasks (in the current runtime)
                let task = self.state().borrow_mut().tasks.remove(task);

                // can be none if drop of revious task canceled this one
                drop(task);
            });

            let state = self.state();
            let mut state = state.borrow_mut();

            // remove tasks from pending
            let mut v = Default::default();
            std::mem::swap(&mut v, &mut state.pending);
            state.pending = v.into_iter().filter(|id| !to_cancel.contains(id)).collect();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Runtime(Rc<RefCell<RuntimeState>>);

////////////////////////////////////////////////////////////////////////////////

impl Runtime {
    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle(Rc::downgrade(&self.0))
    }

    pub fn process_next_task(&self) -> bool {
        let (task_id, mut task) = {
            let mut state = self.0.borrow_mut();
            state.clear_pending();

            let Some(task_id) = state.pending.pop_front() else {
                return false;
            };
            let Some(task) = state.tasks.remove(&task_id) else {
                // future already resolved
                return true;
            };
            (task_id, task)
        };

        let poll_result = {
            let waker = futures::task::waker(Arc::new(Waker::new(self.handle(), task_id)));
            let mut ctx = futures::task::Context::from_waker(&waker);
            let poll_result = task.future.as_mut().poll(&mut ctx);
            poll_result
        };

        if poll_result.is_pending() {
            self.0.borrow_mut().tasks.insert(task_id, task);
        }

        true
    }

    #[allow(unused)]
    pub fn process_tasks(&self) -> usize {
        let mut processed = 0;
        while self.process_next_task() {
            processed += 1;
        }
        processed
    }

    pub fn next_task_owner(&self) -> Option<ProcessHandle> {
        self.0.borrow_mut().next_task_owner()
    }
}
