use std::{
    cell::{RefCell, RefMut},
    collections::{HashMap, VecDeque},
    future::Future,
    rc::{Rc, Weak},
    sync::Arc,
};

use super::{
    task::{JoinHandle, Task, TaskId},
    waker::Waker,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct State {
    pending: VecDeque<TaskId>,
    tasks: HashMap<TaskId, Task>,
    next_task_id: TaskId,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Handle(Weak<RefCell<State>>);

////////////////////////////////////////////////////////////////////////////////

thread_local! {
    static HANDLE: RefCell<Option<Handle>> = RefCell::new(None);
}

////////////////////////////////////////////////////////////////////////////////

impl Handle {
    fn is_runtime_destroyed(&self) -> bool {
        self.0.strong_count() == 0
    }

    fn state(&self) -> Rc<RefCell<State>> {
        self.0.upgrade().unwrap()
    }

    pub fn schedule(&self, task: TaskId) {
        if !self.is_runtime_destroyed() {
            self.state().borrow_mut().pending.push_back(task);
        }
    }

    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        let binding = self.state();
        let mut state = binding.borrow_mut();

        let task_id = state.next_task_id;
        state.next_task_id += 1;

        let (sender, receiver) = tokio::sync::oneshot::channel();
        let task = async move {
            let result = task.await;
            let _ = sender.send(result); // receiver can be dropped which is ok
        };
        state.tasks.insert(task_id, Box::pin(task));
        state.pending.push_back(task_id);

        JoinHandle::new(task_id, receiver)
    }

    pub fn current() -> Self {
        HANDLE.with(|h| {
            h.borrow()
                .as_ref()
                .expect("must be called in runtime")
                .clone()
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Runtime(Rc<RefCell<State>>);

impl Runtime {
    pub fn handle(&self) -> Handle {
        Handle(Rc::downgrade(&self.0))
    }

    pub fn process_next_task(&self) -> bool {
        let (task_id, mut task) = {
            let mut state = self.0.borrow_mut();
            let Some(task_id) = state.pending.pop_front() else {
                return false;
            };
            let Some(task) = state.tasks.remove(&task_id) else {
                panic!("missing task: {task_id}");
            };
            (task_id, task)
        };

        self.set_current_handle();
        let poll_result = {
            let waker = futures::task::waker(Arc::new(Waker::new(self.handle(), task_id)));
            let mut ctx = futures::task::Context::from_waker(&waker);
            let poll_result = task.as_mut().poll(&mut ctx);
            poll_result
        };
        self.remove_current_handle();

        if poll_result.is_pending() {
            self.0.borrow_mut().tasks.insert(task_id, task);
        }

        true
    }

    pub fn process_tasks(&self) -> usize {
        let mut processed = 0;
        while self.process_next_task() {
            processed += 1;
        }
        processed
    }

    fn set_current_handle(&self) {
        HANDLE.with(|h| {
            *h.borrow_mut() = Some(self.handle());
        });
    }

    fn remove_current_handle(&self) {
        HANDLE.with(|h| {
            *h.borrow_mut() = None;
        });
    }

    pub fn next_task_id(&self) -> Option<TaskId> {
        self.0.borrow().pending.front().copied()
    }

    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
    {
        self.handle().spawn(task)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn spawn<F>(task: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
{
    Handle::current().spawn(task)
}
