use super::{runtime::Handle, task::TaskId};

////////////////////////////////////////////////////////////////////////////////

pub struct Waker {
    handle: Handle,
    task_id: TaskId,
}

impl Waker {
    pub fn new(handle: Handle, task_id: TaskId) -> Self {
        Self { handle, task_id }
    }
}

unsafe impl Sync for Waker {}
unsafe impl Send for Waker {}

impl futures::task::ArcWake for Waker {
    fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
        arc_self.handle.schedule(arc_self.task_id);
    }
}
